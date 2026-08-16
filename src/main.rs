use axum::{routing::get, Router};

async fn hello() -> &'static str {
    "proof-rust: one estate, many LXS. just use it."
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("SERVER_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8500);
    let app = Router::new().route("/", get(hello));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    println!("[proof-rust] listening on :{port}");
    axum::serve(listener, app).await.unwrap();
}
