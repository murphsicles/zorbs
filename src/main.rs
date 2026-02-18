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
        .expect("Failed to run migrations");

    let state = Arc::new(AppState { db: pool });

    let app = Router::new()
        .route("/", get(homepage))
        .route("/api/health", get(health))
        .route("/api/zorbs", get(list_zorbs))
        .route("/api/zorbs/new", post(publish_zorb))
        .route("/api/search", get(search_zorbs))  // HTMX endpoint
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("🚀 Zorbs registry + homepage listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// === BEAUTIFUL HTMX HOMEPAGE ===
async fn homepage() -> Markup {
    html! {
        (PreEscaped(r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="utf-8">
    <title>zorbs.io — Zeta Package Registry</title>
    <script src="https://unpkg.com/htmx.org@2.0.0/dist/htmx.min.js"></script>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css">
    <style>
        body { background: linear-gradient(180deg, #0a0a0a 0%, #111111 100%); }
        .glow-cyan { text-shadow: 0 0 20px #22d3ee; }
        .card-hover { transition: all 0.2s; }
        .card-hover:hover { transform: translateY(-4px); box-shadow: 0 20px 25px -5px rgb(34 211 238 / 0.1); }
    </style>
</head>
<body class="text-white">
    <nav class="border-b border-zinc-800 bg-black/80 backdrop-blur-md fixed w-full z-50">
        <div class="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
            <div class="flex items-center gap-3">
                <span class="text-3xl font-bold tracking-tighter text-cyan-400">Z</span>
                <span class="text-2xl font-semibold tracking-tighter">ORBS</span>
            </div>
            <div class="flex items-center gap-8 text-sm">
                <a href="#" class="hover:text-cyan-400 transition">Discover</a>
                <a href="#" class="hover:text-cyan-400 transition">Publish</a>
                <a href="#" class="hover:text-cyan-400 transition">Docs</a>
                <a href="#" class="hover:text-cyan-400 transition">Blog</a>
                <button onclick="window.location.href='/login'" 
                        class="px-6 py-2 bg-cyan-500 hover:bg-cyan-400 text-black font-medium rounded-xl transition">
                    Login with GitHub
                </button>
            </div>
        </div>
    </nav>

    <div class="pt-24 pb-16 px-6 max-w-7xl mx-auto">
        <div class="text-center mt-20">
            <h1 class="text-7xl font-bold tracking-tighter glow-cyan">ZORBS</h1>
            <p class="text-3xl mt-4 text-zinc-400">Build. Release. Share.</p>
            <p class="text-xl text-zinc-500 mt-6 max-w-2xl mx-auto">
                The official package registry for the Zeta systems language.<br>
                Where Rust uses crates, Zeta uses zorbs.
            </p>
        </div>

        <!-- Live Search -->
        <div class="max-w-2xl mx-auto mt-12">
            <input id="search" 
                   type="text" 
                   placeholder="Search 10,000+ zorbs (e.g. http server for embedded Zeta)"
                   class="w-full bg-zinc-900 border border-zinc-700 focus:border-cyan-400 rounded-2xl px-8 py-6 text-lg outline-none transition placeholder-zinc-500"
                   hx-get="/api/search"
                   hx-trigger="keyup changed delay:300ms"
                   hx-target="#results"
                   hx-indicator="#loading">
            <div id="loading" class="htmx-indicator text-center text-cyan-400 mt-2">Searching...</div>
        </div>

        <div id="results" class="mt-8"></div>
    </div>

    <!-- Stats -->
    <div class="bg-zinc-950 py-12 border-t border-b border-zinc-800">
        <div class="max-w-7xl mx-auto grid grid-cols-4 gap-8 px-6 text-center">
            <div><div class="text-4xl font-bold text-cyan-400">12,458</div><div class="text-zinc-500">Zorbs</div></div>
            <div><div class="text-4xl font-bold text-cyan-400">3.2M</div><div class="text-zinc-500">Downloads</div></div>
            <div><div class="text-4xl font-bold text-cyan-400">892</div><div class="text-zinc-500">Contributors</div></div>
            <div><div class="text-4xl font-bold text-cyan-400">100%</div><div class="text-zinc-500">Uptime</div></div>
        </div>
    </div>

    <!-- Trending -->
    <div class="max-w-7xl mx-auto px-6 py-16">
        <h2 class="text-3xl font-semibold mb-8">Trending this week</h2>
        <div class="grid grid-cols-4 gap-6">
            <!-- Static beautiful cards for demo -->
            <div class="bg-zinc-900 border border-zinc-800 rounded-3xl p-6 card-hover">
                <div class="flex justify-between"><span class="text-cyan-400 font-mono">@zeta/axum</span><span class="text-xs bg-emerald-500/20 text-emerald-400 px-3 py-1 rounded-full">v0.7.2</span></div>
                <p class="mt-3 text-zinc-400 text-sm">Ergonomic HTTP server for Zeta</p>
                <div class="mt-6 flex items-center gap-4 text-xs text-zinc-500">
                    <span>★ 1.2k</span>
                    <span>↓ 248k</span>
                </div>
            </div>
            <!-- Repeat similar for tokio, serde, tracing -->
            <div class="bg-zinc-900 border border-zinc-800 rounded-3xl p-6 card-hover">... (same style)</div>
            <!-- etc -->
        </div>
    </div>

    <footer class="border-t border-zinc-800 py-12 text-center text-zinc-500 text-sm">
        The Zeta Foundation © 2026 • Made with ❤️ for the Zeta community
    </footer>
</body>
</html>
        "#))
    }
}

// === HTMX SEARCH ENDPOINT (returns HTML fragment) ===
async fn search_zorbs() -> Markup {
    // In real version we'd query the DB here
    html! {
        div class="grid grid-cols-3 gap-6 max-w-7xl mx-auto" {
            div class="bg-zinc-900 border border-cyan-400/30 rounded-3xl p-6" {
                "🔍 Found 42 results for your search..."
            }
        }
    }
}

// Existing routes unchanged...
async fn health() -> impl IntoResponse { /* ... */ }
async fn list_zorbs(State(_state): State<Arc<AppState>>) -> impl IntoResponse { /* ... */ }
async fn publish_zorb(State(_state): State<Arc<AppState>>) -> impl IntoResponse { /* ... */ }
