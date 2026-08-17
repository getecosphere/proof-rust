//! Notes — the estate core owns its own domain data.
//!
//! A MongoDB-backed note CRUD for the signed-in user (proves the estate is
//! data-driven: identity comes from the auth LXS, the data lives in Mongo, and
//! the core owns only this app-specific domain). Also hosts the `user.signed_up`
//! event bridge: auth emits the event to `/api/events/signup`, this core
//! validates the service token, then forwards it to the notifications LXS
//! ingest so a signup becomes an in-app notification.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use bson::oid::ObjectId;
use futures_util::{StreamExt, TryStreamExt};
use mongodb::{
    bson::doc,
    options::IndexOptions,
    IndexModel,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct NotesApi {
    pub collection: mongodb::Collection<Note>,
    pub jwt_secret: String,
    pub notifications_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub pinned: bool,
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
}

impl Note {
    pub fn public_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id.map(|o| o.to_string()),
            "title": self.title,
            "body": self.body,
            "pinned": self.pinned,
            "createdAt": self.created_at.to_chrono().to_rfc3339(),
            "updatedAt": self.updated_at.to_chrono().to_rfc3339(),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInput {
    pub title: Option<String>,
    pub body: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(Deserialize)]
pub struct SignupEventPayload {
    #[serde(rename = "event")]
    pub event: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
}

/// Read the authenticated user id from the gateway-injected `X-Eco-User`
/// header (`<sub>,roles=<role>`). Client-supplied headers are stripped by the
/// gateway before forwarding, so this is trustworthy on auth-level routes.
pub fn user_from_headers(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("x-eco-user")?.to_str().ok()?;
    let sub = value.split(',').next().unwrap_or("").trim();
    if sub.is_empty() {
        None
    } else {
        Some(sub.to_string())
    }
}

/// Validate an HS512 bearer token with the estate's shared JWT_SECRET.
fn verify_service_token(secret: &str, headers: &HeaderMap) -> bool {
    let Some(value) = headers.get("authorization") else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };
    let Some(token) = value.strip_prefix("Bearer ").or_else(|| value.strip_prefix("bearer ")) else {
        return false;
    };
    jsonwebtoken::decode::<serde_json::Value>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512),
    )
    .is_ok()
}

pub async fn list_notes(State(api): State<NotesApi>, headers: HeaderMap) -> axum::response::Response {
    let Some(user_id) = user_from_headers(&headers) else {
        return unauthorized();
    };
    let filter = doc! { "userId": user_id };
    let options = mongodb::options::FindOptions::builder()
        .sort(doc! { "pinned": -1i32, "updatedAt": -1i32 })
        .build();
    let mut cursor = match api.collection.find(filter, options).await {
        Ok(c) => c,
        Err(e) => return internal(format!("list notes: {e}")),
    };
    let mut notes: Vec<serde_json::Value> = Vec::new();
    while let Some(note) = cursor.try_next().await.ok().flatten() {
        notes.push(note.public_json());
    }
    Json(notes).into_response()
}

pub async fn create_note(
    State(api): State<NotesApi>,
    headers: HeaderMap,
    Json(input): Json<NoteInput>,
) -> axum::response::Response {
    let Some(user_id) = user_from_headers(&headers) else {
        return unauthorized();
    };
    let now = bson::DateTime::from_chrono(chrono::Utc::now());
    let note = Note {
        id: None,
        user_id: user_id.clone(),
        title: input.title.unwrap_or_default().trim().to_string(),
        body: input.body.unwrap_or_default(),
        pinned: input.pinned.unwrap_or(false),
        created_at: now,
        updated_at: now,
    };
    match api.collection.insert_one(&note, None).await {
        Ok(result) => {
            let mut created = note;
            created.id = result.inserted_id.as_object_id();
            Json(created.public_json()).into_response()
        }
        Err(e) => internal(format!("create note: {e}")),
    }
}

pub async fn update_note(
    State(api): State<NotesApi>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<NoteInput>,
) -> axum::response::Response {
    let Some(user_id) = user_from_headers(&headers) else {
        return unauthorized();
    };
    let Ok(oid) = ObjectId::parse_str(&id) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" }))).into_response();
    };
    let Some(existing) = api
        .collection
        .find_one(doc! { "_id": oid, "userId": &user_id }, None)
        .await
        .ok()
        .flatten()
    else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" }))).into_response();
    };
    let title = input.title.unwrap_or(existing.title);
    let body = input.body.unwrap_or(existing.body);
    let pinned = input.pinned.unwrap_or(existing.pinned);
    let now = bson::DateTime::from_chrono(chrono::Utc::now());
    match api
        .collection
        .update_one(
            doc! { "_id": oid, "userId": &user_id },
            doc! {
                "$set": {
                    "title": &title,
                    "body": &body,
                    "pinned": pinned,
                    "updatedAt": now,
                }
            },
            None,
        )
        .await
    {
        Ok(_) => {
            let updated = Note {
                id: Some(oid),
                user_id,
                title,
                body,
                pinned,
                created_at: existing.created_at,
                updated_at: now,
            };
            Json(updated.public_json()).into_response()
        }
        Err(e) => internal(format!("update note: {e}")),
    }
}

pub async fn delete_note(
    State(api): State<NotesApi>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> axum::response::Response {
    let Some(user_id) = user_from_headers(&headers) else {
        return unauthorized();
    };
    let Ok(oid) = ObjectId::parse_str(&id) else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" }))).into_response();
    };
    match api
        .collection
        .delete_one(doc! { "_id": oid, "userId": user_id }, None)
        .await
    {
        Ok(r) if r.deleted_count == 1 => Json(serde_json::json!({ "ok": true })).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" }))).into_response(),
        Err(e) => internal(format!("delete note: {e}")),
    }
}

/// Auth's `user.signed_up` event bridge. Validates the estate service token,
/// then forwards a translated notification to the notifications LXS ingest.
pub async fn signup_event(
    State(api): State<NotesApi>,
    headers: HeaderMap,
    Json(payload): Json<SignupEventPayload>,
) -> axum::response::Response {
    if !verify_service_token(&api.jwt_secret, &headers) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "invalid token" }))).into_response();
    }
    let email = payload.email.clone().unwrap_or_default();
    let email_label = if email.is_empty() { "—".to_string() } else { email };
    if api.notifications_url.is_empty() {
        return Json(serde_json::json!({ "ok": true, "forwarded": false })).into_response();
    }
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .map(|s| {
            s.strip_prefix("Bearer ")
                .or_else(|| s.strip_prefix("bearer "))
                .unwrap_or(s)
                .to_string()
        })
        .filter(|t| !t.is_empty());
    let url = format!("{}/ingest", api.notifications_url.trim_end_matches('/'));
    let body = serde_json::json!({
        "recipient_ids": [payload.user_id],
        "kind": "system",
        "title": "Welcome to proof-rust",
        "body": format!(
            "Your account ({email_label}) was created. The auth LXS emitted user.signed_up → notifications LXS delivered this."
        ),
        "reference_id": payload.user_id,
    });
    let mut request = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(5));
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    match request.send().await {
        Ok(resp) if resp.status().is_success() => {
            Json(serde_json::json!({ "ok": true, "forwarded": true })).into_response()
        }
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            internal(format!("notifications ingest returned {status}: {text}"))
        }
        Err(e) => internal(format!("notifications ingest failed: {e}")),
    }
}

pub async fn init_indexes(collection: &mongodb::Collection<Note>) -> Result<(), mongodb::error::Error> {
    let model = IndexModel::builder()
        .keys(doc! { "userId": 1i32 })
        .options(IndexOptions::builder().build())
        .build();
    collection.create_index(model, None).await?;
    Ok(())
}

fn unauthorized() -> axum::response::Response {
    (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "unauthorized" }))).into_response()
}

fn internal(message: String) -> axum::response::Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": message }))).into_response()
}
