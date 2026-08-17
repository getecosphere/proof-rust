use axum::{
    body::Body,
    extract::State,
    http::{header::{CACHE_CONTROL, CONTENT_TYPE}, HeaderValue, StatusCode},
    response::Response,
    routing::{get, post, put, delete},
    Router,
};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_axum::render_app_to_stream;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod notes;
use notes::{init_indexes, Note, NotesApi};

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Dashboard,
}

// Self-contained static binary: the stylesheet is embedded, so there is no
// static/ dir to ship and nothing to miss on the CT.
const STYLE_CSS: &str = include_str!("../static/style.css");
const ECO_LOGO: &[u8] = include_bytes!("../static/ecosphere.png");
const ECO_LOGO_MARK: &[u8] = include_bytes!("../static/ecosphere-mark.png");

async fn serve_style() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("text/css; charset=utf-8"))
        .body(Body::from(STYLE_CSS))
        .unwrap()
}

async fn serve_logo() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("image/png"))
        .header(CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400"))
        .body(Body::from(ECO_LOGO))
        .unwrap()
}

async fn serve_logo_mark() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("image/png"))
        .header(CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400"))
        .body(Body::from(ECO_LOGO_MARK))
        .unwrap()
}

fn page_title() -> String {
    "Proof Estate — Rust · composed with the auth LXS".to_string()
}

fn page_meta() -> &'static str {
    "A Rust estate that composes the auth LXS binary: full signin/signup, one shared identity, zero rebuilds on the server."
}

#[component]
fn Header() -> impl IntoView {
    let nav_script = r##"(function () {
      var s = (function () { try { return JSON.parse(localStorage.getItem("eco_session") || "null"); } catch (e) { return null; } })();
      var a = document.getElementById("header-auth");
      if (!a) return;
      if (s && s.token) {
        var name = (s.user && s.user.name) ? s.user.name : "";
        a.innerHTML = '<a class="btn-auth btn-auth-dash" href="/dashboard">Dashboard</a>' +
          '<a class="btn-auth btn-auth-out" href="/signin" data-logout="1">Sign out</a>';
        var out = a.querySelector("[data-logout]");
        if (out) out.addEventListener("click", function (e) { e.preventDefault(); localStorage.removeItem("eco_session"); window.location.href = "/"; });
      } else {
        a.innerHTML = '<a class="btn-auth btn-auth-in" href="/signin">Sign in</a>';
      }
    })();"##;
    view! {
        <header class="top">
            <div class="top-inner">
                <a class="brand" href="/">
                    <img class="brand-mark-img" src="/static/ecosphere-mark.png" alt="Ecosphere" />
                    <span class="brand-name">"Proof Estate"</span>
                </a>
                <nav class="nav" aria-label="Primary">
                    <a href="#lxss">"LXS"</a>
                    <a href="#compose">"Compose"</a>
                    <a href="#proof">"Proof"</a>
                    <span class="header-avatar-wrap" data-name="1">
                        <span id="header-avatar-txt" class="header-avatar">"?"</span>
                        <img id="header-avatar-img" class="header-avatar header-avatar-img" alt="avatar" style="display:none" />
                    </span>
                    <span id="header-auth"></span>
                </nav>
            </div>
        </header>
        <script>{nav_script}</script>
    }
}

#[component]
fn Hero() -> impl IntoView {
    view! {
        <section class="hero shell">
            <div class="hero-copy">
                <p class="kicker">"ECO · LXS · ESTATES"</p>
                <h1>"One estate. Many LXS. Just use it."</h1>
                <p class="lede">"Stop rebuilding the same capability in every app. Auth, storage, notifications, articles — compiled once, composed everywhere."</p>
                <p class="hero-sub">"This is a real estate: a Rust core that composes the auth LXS binary. Sign up, sign in, and the identity is shared by every capability on the estate — no rewritten auth, no duplicated credentials."</p>
                <div class="hero-actions">
                    <a class="btn-primary" href="/signup">"Start composing →"</a>
                    <a class="btn-secondary" href="/signin">"Sign in"</a>
                </div>
            </div>
            <div class="hero-compose" aria-label="Capabilities compose into an estate and become globally reachable">
                <div class="estate-frame">
                    <div class="compose-stage">
                        <span class="cap-node n-auth">"auth"<small>"LXS · Rust"</small></span>
                        <span class="cap-node n-ui">"auth-ui"<small>"LXS · Leptos"</small></span>
                        <span class="cap-node n-core">"proof-rust"<small>"core · axum"</small></span>
                        <span class="cap-node core">"your estate"</span>
                        <i class="rel rel-a" aria-hidden="true"></i>
                        <i class="rel rel-b" aria-hidden="true"></i>
                        <span class="frame-tag" aria-hidden="true">"ESTATE"</span>
                    </div>
                    <div class="compose-state" aria-hidden="true">
                        <span class="state-dot s1">"○ defined"</span>
                        <span class="state-dot s2">"◐ starting"</span>
                        <span class="state-dot s3">"● ready"</span>
                    </div>
                </div>
                <div class="compose-route" aria-hidden="true">
                    <span class="route-local">"LOCAL"</span>
                    <i class="route-line"></i>
                    <span class="route-global">"GLOBALLY ACCESSIBLE"</span>
                </div>
            </div>
        </section>
    }
}

#[component]
fn LxsNarrative() -> impl IntoView {
    view! {
        <section class="section manifesto" id="lxss" data-reveal>
            <p class="kicker">"WHAT IS AN LXS?"</p>
            <h2 class="manifesto-line">"A compiled capability is a product. Not a codebase."</h2>
            <p class="large-copy manifesto-copy">"A Linux Service (LXS) is a single static binary that owns one bounded domain — auth, storage, notifications, articles. It ships with a declared contract: what env it needs, what it provides, what it touches. No source to vendor, no runtime to install, no build step on the server."</p>
            <div class="lxs-def">
                <ul>
                    <li>"One compiled binary — Rust, musl, self-contained"</li>
                    <li>"One bounded domain — auth owns identity, nothing else"</li>
                    <li>"A declared contract — env, database, network, resources"</li>
                    <li>"Self-sufficient — no language runtime, no framework on the host"</li>
                </ul>
                <p><b>"● ready"</b> " — the auth LXS on this estate is a real binary from the registry. Sign in below and feel what composed identity is like."</p>
            </div>
        </section>
    }
}

#[component]
fn ComposeSection() -> impl IntoView {
    view! {
        <section class="section compose-proof" id="compose" data-reveal>
            <div class="section-grid">
                <div>
                    <p class="kicker">"HOW IT WORKS"</p>
                    <h2>"Compose, don't rebuild."</h2>
                    <p class="large-copy">"The estate core stays small and yours. Identity is pulled in as a versioned LXS binary — the same one every estate uses — while the code you actually own stays focused."</p>
                </div>
                <div class="proof-band-compact">
                    <div class="cmd-line"><code>eco init</code><span>"detect this Rust core"</span></div>
                    <div class="cmd-line"><code>eco lxs add auth@1.1.0</code><span>"compose the auth binary"</span></div>
                    <div class="cmd-line"><code>eco up dev</code><span>"run locally"</span></div>
                    <div class="cmd-line"><code>eco up --remote</code><span>"ship executable to Ecosphere"</span></div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn ProofSection() -> impl IntoView {
    view! {
        <section class="section proof-section" id="proof" data-reveal>
            <p class="kicker">"THE PROOF"</p>
            <h2>"One binary. Every framework."</h2>
            <p>"The same auth​@1.1.0 binary powers this estate and nine proof estates — Rust, Go, Spring Boot, Next.js, Vite, Astro, Nuxt, Node, static. Register here, then the identity is real: a JWT issued by a 10.7 MB binary that never compiled on the server."</p>
            <div class="stress-evidence"><strong>"10.7 MB"<small>"auth LXS linux binary"</small></strong><span>"25.0 MB before avatar/storage were removed"</span><i>"→"</i><span class="stress-success">"pure identity, still 57% lighter"</span></div>
            <a class="text-link" href="/signup">"Try the live signup →"</a>
        </section>
    }
}

#[component]
fn PoweredBy() -> impl IntoView {
    // Rust crab mascot as a real Leptos view node (not an escaped string).
    let crab = view! {
        <svg class="crab" viewBox="0 0 64 64" aria-hidden="true">
            <g fill="currentColor">
                <ellipse cx="32" cy="38" rx="17" ry="15"/>
                <circle cx="22" cy="32" r="4"/><circle cx="42" cy="32" r="4"/>
                <circle cx="22" cy="31" r="1.6" fill="#f7f6f2"/><circle cx="42" cy="31" r="1.6" fill="#f7f6f2"/>
                <rect x="29" y="42" width="6" height="5" rx="2"/>
                <path d="M8 30 Q20 14 34 18 L28 26 Z"/>
                <path d="M56 30 Q44 14 30 18 L36 26 Z"/>
                <path d="M6 28 q-6-4 0-10 q8 0 6 8 Z"/>
                <path d="M58 28 q6-4 0-10 q-8 0-6 8 Z"/>
                <rect x="12" y="40" width="10" height="6" rx="3" transform="rotate(-18 17 43)"/>
                <rect x="42" y="40" width="10" height="6" rx="3" transform="rotate(18 47 43)"/>
            </g>
        </svg>
    }
    .into_any();
    view! {
        <section class="section powered-by" data-reveal>
            <p class="kicker">"POWERED BY"</p>
            <h2>"Built on real, composable technology."</h2>
            <div class="power-grid">
                <a class="power-card" href="https://www.rust-lang.org/" target="_blank" rel="noopener">
                    {crab}
                    <span class="power-name">"Rust"</span>
                    <span class="power-desc">"This estate's core is a Rust (axum + Leptos) service. The auth LXS is a single static Rust binary."</span>
                    <span class="power-link">"rust-lang.org →"</span>
                </a>
                <a class="power-card" href="https://leptos.dev/" target="_blank" rel="noopener">
                    <span class="power-logo leptos">"L"</span>
                    <span class="power-name">"Leptos"</span>
                    <span class="power-desc">"Homepage and auth-ui pages are server-rendered with Leptos — fine-grained reactive Rust for the web."</span>
                    <span class="power-link">"leptos.dev →"</span>
                </a>
                <a class="power-card" href="https://getecosphere.com/" target="_blank" rel="noopener">
                    <span class="power-logo eco"><img class="power-logo-img" src="/static/ecosphere.png" alt="Ecosphere logo" /></span>
                    <span class="power-name">"Ecosphere"</span>
                    <span class="power-desc">"The composition platform: one ecompose.yml, one command, binaries shipped and run on Ecosphere compute."</span>
                    <span class="power-link">"getecosphere.com →"</span>
                </a>
            </div>
        </section>
    }
}

#[component]
fn DashboardPage() -> impl IntoView {
    // Client-side guard: no session → redirect to /signin. The Notes workspace
    // itself (two-pane, motion/Apple-style) is the React bundle at
    // /static/notes/notes.js, built on the dev machine and served by this core.
    let dash_js = r##"(function () {
      var s = (function () { try { return JSON.parse(localStorage.getItem("eco_session") || "null"); } catch (e) { return null; } })();
      if (!s || !s.token) { window.location.href = "/signin"; return; }
      var name = (s.user && s.user.name) || "there";
      var greeting = document.getElementById("dash-greeting");
      var sub = document.getElementById("dash-sub");
      if (greeting) greeting.textContent = name;
      if (sub && s.user && s.user.email) sub.textContent = s.user.email;

      // Header avatar top-right (also used on other pages).
      var av = document.getElementById("header-avatar");
      if (av && av.dataset.name) {
        var initials = (name || "?").split(/\s+/).map(function (w) { return w[0]; }).join("").slice(0,2).toUpperCase();
        av.textContent = initials;
      }
      var userId = s.user && s.user.id;
      if (userId) {
        fetch("/api/users/" + encodeURIComponent(userId), { headers: { Authorization: "Bearer " + s.token } })
          .then(function (r) { return r.ok ? r.json() : null; })
          .then(function (p) {
            if (p && p.avatarUrl) {
              var img = document.getElementById("header-avatar-img");
              if (img) { img.src = p.avatarUrl; img.style.display = "inline-block"; }
              var txt = document.getElementById("header-avatar-txt");
              if (txt) txt.style.display = "none";
            }
          }).catch(function () {});
      }
    })();"##;
    view! {
        <section class="dashboard-hero" id="dashboard">
            <p class="kicker">"YOUR ESTATE · PROTECTED"</p>
            <h1 class="dash-title">"Welcome back, "<span id="dash-greeting">"there"</span></h1>
            <p id="dash-sub" class="dash-sub">""</p>
            <p class="dash-lead">"Protected area of a composed estate. Identity came from the auth LXS; your notes are app data this core owns in MongoDB (rendered with motion, Apple-style); notifications — including the user.signed_up event — come from the notifications LXS."</p>
            <div class="dash-actions">
                <a class="btn-primary" href="/profile">"Edit profile →"</a>
                <a class="btn-secondary" href="/">"Back to homepage"</a>
            </div>
        </section>
        <div id="eco-notes-root"></div>
        <script src="/static/notes/notes.js"></script>
        <script>{dash_js}</script>
    }
}

#[component]
fn FinalCta() -> impl IntoView {
    view! {
        <section class="final-cta" data-reveal>
            <h2>"Your estate, composed — not rewritten."</h2>
            <a class="btn-primary" href="/signup">"Sign up on this estate →"</a>
            <a class="btn-secondary" href="/signin">"Or sign in"</a>
        </section>
    }
}

#[component]
fn PlumbingSection() -> impl IntoView {
    // The estate manifest: zero auth code, zero UI code — pure composition.
    let ecompose = r#"project: proof-rust

estates:
  proof-rust:
    hostname: proof-rust.getecosphere.com
    ingress: tunnel
    cloudflare_account: getecosphere
    services:
      - proof-rust
      - auth-backend
      - auth-ui

services:
  proof-rust:
    path: .
    runtimes:
      - rust

  auth-backend:
    lxs: auth​@1.1.0

  auth-ui:
    lxs: auth-ui​@0.1.0

auth:
  email_verification:
    enabled: false"#;
    let fetch_script = r##"(function () {
      var pre = document.getElementById("ecompose-live");
      if (!pre) return;
      fetch("https://raw.githubusercontent.com/getecosphere/proof-rust/main/ecompose.yml")
        .then(function (r) { if (!r.ok) throw new Error(r.status); return r.text(); })
        .then(function (t) { pre.textContent = t; pre.classList.add("loaded"); })
        .catch(function () { /* keep the embedded copy */ });
    })();"##;
    view! {
        <section class="section plumbing-section" id="plumbing" data-reveal>
            <div class="section-grid">
                <div>
                    <p class="kicker">"THE PLUMBING"</p>
                    <h2>"Zero auth code. Zero UI code. Just composition."</h2>
                    <p class="large-copy">"There is no auth implementation in this estate — not the API, not the signin page. The whole identity boundary is two lines that say <em>compose the auth LXS</em>. Everything else (JWT, registration, verification, the signin/signup UI) is a binary pulled from the registry and run as-is."</p>
                    <ul class="plumbing-list">
                        <li><b>auth-backend</b><span>"the auth API — login, register, JWT, verify — a 10.7 MB binary, zero source here"</span></li>
                        <li><b>auth-ui</b><span>"the signin/signup pages — a 1.3 MB binary, zero source here"</span></li>
                        <li><b>proof-rust</b><span>"the only code this estate owns — the homepage you are reading"</span></li>
                    </ul>
                    <div class="git-card">
                        <svg class="git-mark" viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>
                        <div class="git-copy">
                            <b>getecosphere/proof-rust</b>
                            <code>"git clone https://github.com/getecosphere/proof-rust.git"</code>
                        </div>
                        <a class="btn-secondary git-btn" href="https://github.com/getecosphere/proof-rust" target="_blank" rel="noopener">"View on GitHub →"</a>
                    </div>
                </div>
                <div class="code-shell">
                    <div class="code-head"><span class="dot dot-r"></span><span class="dot dot-y"></span><span class="dot dot-g"></span><code>ecompose.yml</code></div>
                    <pre id="ecompose-live" class="code-pre">{ecompose}</pre>
                </div>
            </div>
        </section>
        <script>{fetch_script}</script>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="foot"><div class="foot-inner"><p>"Proof estate · Rust + auth LXS · composed with eco"</p><div class="foot-nav"><a href="/signin">"Sign in"</a><a href="/signup">"Sign up"</a></div></div></footer>
    }
}

#[component]
fn App(page: Page) -> impl IntoView {
    let theme_script = r##"(function () { document.documentElement.setAttribute("data-theme", "light"); })();"##;
    let reveal_script = r##"(function () {
      var els = document.querySelectorAll("[data-reveal]");
      if (!("IntersectionObserver" in window)) { els.forEach(function (e) { e.classList.add("in-view"); }); return; }
      var io = new IntersectionObserver(function (entries) {
        entries.forEach(function (en) { if (en.isIntersecting) { en.target.classList.add("in-view"); io.unobserve(en.target); } });
      }, { threshold: 0.12 });
      els.forEach(function (e) { io.observe(e); });
    })();"##;
    let body = if page == Page::Dashboard {
        view! { <DashboardPage /> }.into_any()
    } else {
        view! {
            <Hero />
            <LxsNarrative />
            <ComposeSection />
            <PlumbingSection />
            <ProofSection />
            <PoweredBy />
            <FinalCta />
        }.into_any()
    };
    view! {
        <html data-theme="light" lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="description" content={page_meta()} />
                <title>{page_title()}</title>
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true" />
                <link href="https://fonts.googleapis.com/css2?family=DM+Mono:wght@400;500&family=Manrope:wght@400;500;600;700;800&display=swap" rel="stylesheet" />
                <link rel="stylesheet" href="/static/style.css?v=20260816d" />
                <script>{theme_script}</script>
            </head>
            <body>
                <Header />
                {body}
                <Footer />
                <script>{reveal_script}</script>
            </body>
        </html>
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .or_else(|| std::env::var("SERVER_PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(8500);
    let notes_api = build_notes_api().await;
    let app = Router::new()
        .route("/", get(render_app_to_stream(|| view! { <App page=Page::Home /> })))
        .route("/dashboard", get(render_app_to_stream(|| view! { <App page=Page::Dashboard /> })))
        .route("/static/style.css", get(serve_style))
        .route("/static/ecosphere.png", get(serve_logo))
        .route("/static/ecosphere-mark.png", get(serve_logo_mark))
        .nest_service("/static/notes", ServeDir::new("static/notes"))
        .route("/api/notes", get(notes::list_notes).post(notes::create_note))
        .route("/api/notes/{id}", put(notes::update_note).delete(notes::delete_note))
        .route("/api/events/signup", post(notes::signup_event))
        .with_state(notes_api);
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("proof-rust frontend could not bind its port");
    println!("proof-rust frontend listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await.expect("proof-rust frontend stopped unexpectedly");
}

async fn build_notes_api() -> NotesApi {
    let uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017/proof_rust_proof_rust".to_string());
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_default();
    let notifications_url = std::env::var("NOTIFICATIONS_URL").unwrap_or_default();
    let notes_api = match mongodb::Client::with_uri_str(&uri).await {
        Ok(client) => {
            let db = client
                .default_database()
                .unwrap_or_else(|| client.database("proof_rust_proof_rust"));
            let collection = db.collection::<Note>("notes");
            let _ = init_indexes(&collection).await;
            NotesApi {
                collection,
                jwt_secret,
                notifications_url,
            }
        }
        Err(e) => {
            eprintln!("notes: mongodb unavailable ({e}); notes + signup bridge disabled");
            let collection = mongodb::Client::with_uri_str("mongodb://localhost:27017")
                .await
                .expect("mongo client")
                .database("eco")
                .collection::<Note>("notes");
            NotesApi {
                collection,
                jwt_secret,
                notifications_url,
            }
        }
    };
    notes_api
}
