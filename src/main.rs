use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_axum::render_app_to_stream;
use tokio::net::TcpListener;

// Self-contained static binary: the stylesheet is embedded, so there is no
// static/ dir to ship and nothing to miss on the CT.
const STYLE_CSS: &str = include_str!("../static/style.css");

async fn serve_style() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("text/css; charset=utf-8"))
        .body(Body::from(STYLE_CSS))
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
      var s = (function () { try { return JSON.parse(localStorage.getItem("proof_session") || "null"); } catch (e) { return null; } })();
      var a = document.getElementById("header-auth");
      if (!a) return;
      if (s && s.token) {
        a.innerHTML = '<a class="btn-auth btn-auth-dash" href="/signin" data-logout="1">Sign out</a>';
        var out = a.querySelector("[data-logout]");
        if (out) out.addEventListener("click", function (e) { e.preventDefault(); localStorage.removeItem("proof_session"); window.location.href = "/"; });
      } else {
        a.innerHTML = '<a class="btn-auth btn-auth-in" href="/signin">Sign in</a>';
      }
    })();"##;
    view! {
        <header class="top">
            <div class="top-inner">
                <a class="brand" href="/">
                    <span class="brand-mark" aria-hidden="true">"P"</span>
                    <span class="brand-name">"Proof Estate"</span>
                </a>
                <nav class="nav" aria-label="Primary">
                    <a href="#lxss">"LXS"</a>
                    <a href="#compose">"Compose"</a>
                    <a href="#proof">"Proof"</a>
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
            <p>"The same auth@1.1.0 binary powers this estate and nine proof estates — Rust, Go, Spring Boot, Next.js, Vite, Astro, Nuxt, Node, static. Register here, then the identity is real: a JWT issued by a 10.7 MB binary that never compiled on the server."</p>
            <div class="stress-evidence"><strong>"10.7 MB"<small>"auth LXS linux binary"</small></strong><span>"25.0 MB before avatar/storage were removed"</span><i>"→"</i><span class="stress-success">"pure identity, still 57% lighter"</span></div>
            <a class="text-link" href="/signup">"Try the live signup →"</a>
        </section>
    }
}

#[component]
fn PoweredBy() -> impl IntoView {
    // Rust crab mascot (inline SVG) + official site link.
    let crab = r##"<svg class="crab" viewBox="0 0 64 64" aria-hidden="true"><g fill="currentColor"><ellipse cx="32" cy="38" rx="17" ry="15"/><circle cx="22" cy="32" r="4"/><circle cx="42" cy="32" r="4"/><circle cx="22" cy="31" r="1.6" fill="#f7f6f2"/><circle cx="42" cy="31" r="1.6" fill="#f7f6f2"/><rect x="29" y="42" width="6" height="5" rx="2"/><path d="M8 30 Q20 14 34 18 L28 26 Z"/><path d="M56 30 Q44 14 30 18 L36 26 Z"/><path d="M6 28 q-6-4 0-10 q8 0 6 8 Z"/><path d="M58 28 q6-4 0-10 q-8 0-6 8 Z"/><rect x="12" y="40" width="10" height="6" rx="3" transform="rotate(-18 17 43)"/><rect x="42" y="40" width="10" height="6" rx="3" transform="rotate(18 47 43)"/></g></svg>"##;
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
                    <span class="power-logo eco">"E"</span>
                    <span class="power-name">"Ecosphere"</span>
                    <span class="power-desc">"The composition platform: one ecompose.yml, one command, binaries shipped and run on Ecosphere compute."</span>
                    <span class="power-link">"getecosphere.com →"</span>
                </a>
            </div>
        </section>
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
fn Footer() -> impl IntoView {
    view! {
        <footer class="foot"><div class="foot-inner"><p>"Proof estate · Rust + auth LXS · composed with eco"</p><div class="foot-nav"><a href="/signin">"Sign in"</a><a href="/signup">"Sign up"</a></div></div></footer>
    }
}

#[component]
fn App() -> impl IntoView {
    let theme_script = r##"(function () { document.documentElement.setAttribute("data-theme", "light"); })();"##;
    let reveal_script = r##"(function () {
      var els = document.querySelectorAll("[data-reveal]");
      if (!("IntersectionObserver" in window)) { els.forEach(function (e) { e.classList.add("in-view"); }); return; }
      var io = new IntersectionObserver(function (entries) {
        entries.forEach(function (en) { if (en.isIntersecting) { en.target.classList.add("in-view"); io.unobserve(en.target); } });
      }, { threshold: 0.12 });
      els.forEach(function (e) { io.observe(e); });
    })();"##;
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
                <link rel="stylesheet" href="/static/style.css" />
                <script>{theme_script}</script>
            </head>
            <body>
                <Header />
                <Hero />
                <LxsNarrative />
                <ComposeSection />
                <ProofSection />
                <PoweredBy />
                <FinalCta />
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
    let app = Router::new()
        .route("/", get(render_app_to_stream(|| view! { <App /> })))
        .route("/static/style.css", get(serve_style));
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("proof-rust frontend could not bind its port");
    println!("proof-rust frontend listening on http://0.0.0.0:{port}");
    axum::serve(listener, app).await.expect("proof-rust frontend stopped unexpectedly");
}
