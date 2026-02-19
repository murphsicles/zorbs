use axum::{
    routing::{get, post},
    Router, Json, extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use maud::{html, Markup, PreEscaped};

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/", get(homepage))
        .route("/api/health", get(health))
        .route("/api/zorbs", get(list_zorbs))
        .route("/api/zorbs/new", post(publish_zorb))
        .route("/api/search", get(search_zorbs))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("🚀 Zorbs registry v{} listening on http://localhost:3000", env!("CARGO_PKG_VERSION"));

    axum::serve(listener, app).await.unwrap();
}

async fn homepage() -> Markup {
    html! {
        (PreEscaped(r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>zorbs.io — Zeta Package Registry</title>
    <script src="https://unpkg.com/htmx.org@2.0.0/dist/htmx.min.js"></script>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">
    <style>
        body { background: linear-gradient(180deg, #0a0a0a 0%, #111111 100%); }
        .hero-glow { text-shadow: 0 0 40px rgb(34 211 238); }
        .zorb-card { transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
        .zorb-card:hover { transform: translateY(-8px); box-shadow: 0 25px 50px -12px rgb(34 211 238 / 0.25); }
    </style>
</head>
<body class="text-white min-h-screen">
    <!-- Navbar -->
    <nav class="border-b border-zinc-800 bg-black/90 backdrop-blur-lg fixed w-full z-50">
        <div class="max-w-screen-2xl mx-auto px-8 py-5 flex items-center justify-between">
            <div class="flex items-center gap-3">
                <span class="text-4xl font-black tracking-tighter text-cyan-400">Z</span>
                <span class="text-3xl font-semibold tracking-tighter">ORBS</span>
            </div>
            <div class="flex items-center gap-10 text-sm font-medium">
                <a href="#" class="hover:text-cyan-400 transition-colors">Discover</a>
                <a href="#" class="hover:text-cyan-400 transition-colors">Publish</a>
                <a href="#" class="hover:text-cyan-400 transition-colors">Docs</a>
                <a href="#" class="hover:text-cyan-400 transition-colors">Blog</a>
            </div>
            <button class="px-8 py-3 bg-white text-black font-semibold rounded-2xl hover:bg-cyan-400 hover:text-black transition-all flex items-center gap-2">
                <i class="fa-brands fa-github"></i>
                Login with GitHub
            </button>
        </div>
    </nav>

    <div class="pt-28 pb-20">
        <div class="max-w-screen-2xl mx-auto px-8 text-center">
            <h1 class="text-8xl font-black tracking-tighter hero-glow">ZORBS</h1>
            <p class="text-4xl mt-4 text-zinc-300">Build. Release. Share.</p>
            <p class="mt-8 text-xl text-zinc-400 max-w-3xl mx-auto">
                The official package registry for Zeta.<br>
                Where Rust uses crates, Zeta uses zorbs.
            </p>

            <!-- Search -->
            <div class="max-w-3xl mx-auto mt-16">
                <div class="relative">
                    <input 
                        id="search-input"
                        type="text" 
                        placeholder="Search zorbs... (e.g. async runtime, http server, json parser)"
                        class="w-full bg-zinc-900 border border-zinc-700 focus:border-cyan-500 rounded-3xl px-8 py-7 text-lg outline-none transition-all"
                        hx-get="/api/search"
                        hx-trigger="keyup changed delay:250ms"
                        hx-target="#search-results"
                        hx-indicator="#search-loading">
                    <div id="search-loading" class="htmx-indicator absolute right-8 top-1/2 -translate-y-1/2 text-cyan-400">
                        <i class="fa-solid fa-spinner fa-spin"></i>
                    </div>
                </div>
            </div>
        </div>

        <div id="search-results" class="max-w-screen-2xl mx-auto px-8 mt-12"></div>

        <!-- Trending -->
        <div class="max-w-screen-2xl mx-auto px-8 mt-20">
            <h2 class="text-3xl font-semibold mb-10 flex items-center gap-3">
                <span class="text-cyan-400">🔥</span> Trending this week
            </h2>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8">
                    <div class="flex justify-between items-start">
                        <div>
                            <span class="font-mono text-cyan-400">@zeta/tokio</span>
                            <p class="text-zinc-400 mt-2 text-sm">The async runtime that powers Zeta</p>
                        </div>
                        <span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full">v1.42.0</span>
                    </div>
                    <div class="mt-8 text-xs text-zinc-500 flex gap-6">
                        <span>↓ 428k</span>
                        <span>★ 3.8k</span>
                    </div>
                </div>
                
                <div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8">
                    <div class="flex justify-between items-start">
                        <div>
                            <span class="font-mono text-cyan-400">@http/axum</span>
                            <p class="text-zinc-400 mt-2 text-sm">Ergonomic web framework</p>
                        </div>
                        <span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full">v0.8.1</span>
                    </div>
                    <div class="mt-8 text-xs text-zinc-500 flex gap-6">
                        <span>↓ 312k</span>
                        <span>★ 2.9k</span>
                    </div>
                </div>

                <div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8">
                    <div class="flex justify-between items-start">
                        <div>
                            <span class="font-mono text-cyan-400">@data/serde</span>
                            <p class="text-zinc-400 mt-2 text-sm">Fast &amp; safe serialization</p>
                        </div>
                        <span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full">v1.0.210</span>
                    </div>
                    <div class="mt-8 text-xs text-zinc-500 flex gap-6">
                        <span>↓ 289k</span>
                        <span>★ 4.1k</span>
                    </div>
                </div>

                <div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8">
                    <div class="flex justify-between items-start">
                        <div>
                            <span class="font-mono text-cyan-400">@logging/tracing</span>
                            <p class="text-zinc-400 mt-2 text-sm">Structured, performant logging</p>
                        </div>
                        <span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full">v0.2.5</span>
                    </div>
                    <div class="mt-8 text-xs text-zinc-500 flex gap-6">
                        <span>↓ 197k</span>
                        <span>★ 1.7k</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <footer class="border-t border-zinc-800 py-12 text-center text-zinc-500 text-sm">
        Powered by The Zeta Foundation • © 2026 zorbs.io
    </footer>
</body>
</html>
        "#))
    }
}

async fn search_zorbs() -> Markup {
    html! {
        div class="grid grid-cols-1 md:grid-cols-3 gap-6" {
            div class="bg-zinc-900 border border-emerald-500/30 rounded-3xl p-8 text-center" {
                p class="text-emerald-400 font-medium" { "🔍 Live search coming soon..." }
                p class="text-zinc-400 text-sm mt-2" { "Type in the search box above" }
            }
        }
    }
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy", "service": "zorbs-registry"})))
}

async fn list_zorbs(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "zorbs": [],
        "total": 0,
        "message": "Endpoint ready - database connected"
    })))
}

async fn publish_zorb(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "success": true,
        "message": "Zorb received. Full publishing pipeline coming soon."
    })))
}
