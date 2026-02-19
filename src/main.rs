use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Query, Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use maud::{html, Markup, PreEscaped};
use crate::models::Zorb;
use serde::Deserialize;
use tokio::fs;
use uuid::Uuid;

mod models;

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
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
        .route("/publish", get(publish_page))
        .route("/:name", get(zorb_detail))
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
    <nav class="border-b border-zinc-800 bg-black/90 backdrop-blur-lg fixed w-full z-50">
        <div class="max-w-screen-2xl mx-auto px-8 py-5 flex items-center justify-between">
            <div class="flex items-center gap-3">
                <span class="text-4xl font-black tracking-tighter text-cyan-400">Z</span>
                <span class="text-3xl font-semibold tracking-tighter">ORBS</span>
            </div>
            <div class="flex items-center gap-10 text-sm font-medium">
                <a href="/" class="hover:text-cyan-400 transition-colors">Discover</a>
                <a href="/publish" class="hover:text-cyan-400 transition-colors">Publish</a>
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

async fn publish_page() -> Markup {
    html! {
        (PreEscaped(r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Publish — zorbs.io</title>
    <script src="https://unpkg.com/htmx.org@2.0.0/dist/htmx.min.js"></script>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">
    <style>
        body { background: linear-gradient(180deg, #0a0a0a 0%, #111111 100%); }
        .drop-zone { transition: all 0.3s; }
        .drop-zone.dragover { background-color: rgb(34 211 238 / 0.1); border-color: rgb(34 211 238); }
    </style>
</head>
<body class="text-white min-h-screen">
    <nav class="border-b border-zinc-800 bg-black/90 backdrop-blur-lg fixed w-full z-50">
        <div class="max-w-screen-2xl mx-auto px-8 py-5 flex items-center justify-between">
            <div class="flex items-center gap-3">
                <span class="text-4xl font-black tracking-tighter text-cyan-400">Z</span>
                <span class="text-3xl font-semibold tracking-tighter">ORBS</span>
            </div>
            <div class="flex items-center gap-10 text-sm font-medium">
                <a href="/" class="hover:text-cyan-400 transition-colors">Discover</a>
                <a href="/publish" class="text-cyan-400 font-semibold">Publish</a>
                <a href="#" class="hover:text-cyan-400 transition-colors">Docs</a>
                <a href="#" class="hover:text-cyan-400 transition-colors">Blog</a>
            </div>
            <button class="px-8 py-3 bg-white text-black font-semibold rounded-2xl hover:bg-cyan-400 hover:text-black transition-all flex items-center gap-2">
                <i class="fa-brands fa-github"></i>
                Login with GitHub
            </button>
        </div>
    </nav>

    <div class="pt-28 max-w-2xl mx-auto px-6">
        <div class="text-center mb-12">
            <h1 class="text-6xl font-black tracking-tighter">Publish a Zorb</h1>
            <p class="text-xl text-zinc-400 mt-4">Share your Zeta library with the world in seconds</p>
        </div>

        <form id="publish-form" 
              hx-post="/api/zorbs/new" 
              hx-swap="outerHTML" 
              class="bg-zinc-900 border border-zinc-800 rounded-3xl p-10 space-y-8">

            <div>
                <label class="block text-sm font-medium text-zinc-400 mb-2">Package Name</label>
                <input type="text" name="name" required placeholder="@myteam/my-awesome-zorb" 
                       class="w-full bg-black border border-zinc-700 focus:border-cyan-400 rounded-2xl px-6 py-4 text-lg outline-none">
            </div>

            <div class="grid grid-cols-2 gap-6">
                <div>
                    <label class="block text-sm font-medium text-zinc-400 mb-2">Version</label>
                    <input type="text" name="version" required placeholder="0.1.0" 
                           class="w-full bg-black border border-zinc-700 focus:border-cyan-400 rounded-2xl px-6 py-4 text-lg outline-none">
                </div>
                <div>
                    <label class="block text-sm font-medium text-zinc-400 mb-2">License</label>
                    <select name="license" class="w-full bg-black border border-zinc-700 focus:border-cyan-400 rounded-2xl px-6 py-4 text-lg outline-none">
                        <option value="MIT">MIT</option>
                        <option value="Apache-2.0">Apache-2.0</option>
                        <option value="MIT OR Apache-2.0">MIT OR Apache-2.0</option>
                        <option value="GPL-3.0">GPL-3.0</option>
                    </select>
                </div>
            </div>

            <div>
                <label class="block text-sm font-medium text-zinc-400 mb-2">Description</label>
                <textarea name="description" rows="4" placeholder="A blazing fast HTTP server for Zeta..." 
                          class="w-full bg-black border border-zinc-700 focus:border-cyan-400 rounded-3xl px-6 py-4 text-lg outline-none resize-none"></textarea>
            </div>

            <div>
                <label class="block text-sm font-medium text-zinc-400 mb-2">Repository URL (optional)</label>
                <input type="text" name="repository" placeholder="https://github.com/you/my-zorb" 
                       class="w-full bg-black border border-zinc-700 focus:border-cyan-400 rounded-2xl px-6 py-4 text-lg outline-none">
            </div>

            <div>
                <label class="block text-sm font-medium text-zinc-400 mb-3">Zorb File (.zorb or .tar.gz)</label>
                <div id="drop-zone" 
                     class="drop-zone border-2 border-dashed border-zinc-700 hover:border-cyan-400 rounded-3xl p-12 text-center cursor-pointer"
                     onclick="document.getElementById('file').click()">
                    <i class="fa-solid fa-cloud-upload-alt text-6xl text-cyan-400 mb-4"></i>
                    <p class="text-lg">Drag & drop your .zorb file here</p>
                    <p class="text-sm text-zinc-500 mt-2">or click to browse</p>
                    <input type="file" id="file" name="file" accept=".zorb,.tar.gz" class="hidden">
                </div>
            </div>

            <button type="submit" 
                    class="w-full bg-cyan-400 hover:bg-cyan-300 text-black font-bold py-6 rounded-3xl text-xl transition-all flex items-center justify-center gap-3">
                <i class="fa-solid fa-rocket"></i>
                Publish to zorbs.io
            </button>
        </form>

        <div id="publish-result" class="mt-8"></div>
    </div>

    <footer class="border-t border-zinc-800 py-12 text-center text-zinc-500 text-sm mt-20">
        Powered by The Zeta Foundation • © 2026 zorbs.io
    </footer>

    <script>
        const dropZone = document.getElementById('drop-zone');
        const fileInput = document.getElementById('file');

        dropZone.addEventListener('dragover', e => { e.preventDefault(); dropZone.classList.add('dragover'); });
        dropZone.addEventListener('dragleave', () => dropZone.classList.remove('dragover'));
        dropZone.addEventListener('drop', e => {
            e.preventDefault();
            dropZone.classList.remove('dragover');
            if (e.dataTransfer.files.length) fileInput.files = e.dataTransfer.files;
        });
    </script>
</body>
</html>
        "#))
    }
}

async fn zorb_detail(Path(name): Path<String>, State(state): State<Arc<AppState>>) -> Markup {
    let versions: Vec<Zorb> = sqlx::query_as(
        "SELECT id, name, version, description, license, repository, downloads, created_at 
         FROM zorbs 
         WHERE name = $1 
         ORDER BY created_at DESC"
    )
    .bind(&name)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if versions.is_empty() {
        return html! {
            (PreEscaped(r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <title>Not Found — zorbs.io</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-black text-white min-h-screen flex items-center justify-center">
    <div class="text-center">
        <h1 class="text-8xl font-black text-cyan-400">404</h1>
        <p class="text-3xl mt-8">Zorb not found</p>
        <a href="/" class="mt-12 inline-block px-10 py-4 bg-white text-black rounded-2xl font-medium">Back to Discover</a>
    </div>
</body>
</html>
            "#))
        };
    }

    let latest = &versions[0];

    html! {
        (PreEscaped(r#"
<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>"#)) (name) (PreEscaped(r#" — zorbs.io</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">
    <style>
        body { background: linear-gradient(180deg, #0a0a0a 0%, #111111 100%); }
    </style>
</head>
<body class="text-white min-h-screen">
    <nav class="border-b border-zinc-800 bg-black/90 backdrop-blur-lg fixed w-full z-50">
        <div class="max-w-screen-2xl mx-auto px-8 py-5 flex items-center justify-between">
            <div class="flex items-center gap-3">
                <span class="text-4xl font-black tracking-tighter text-cyan-400">Z</span>
                <span class="text-3xl font-semibold tracking-tighter">ORBS</span>
            </div>
            <div class="flex items-center gap-10 text-sm font-medium">
                <a href="/" class="hover:text-cyan-400 transition-colors">Discover</a>
                <a href="/publish" class="hover:text-cyan-400 transition-colors">Publish</a>
                <a href="#" class="hover:text-cyan-400 transition-colors">Docs</a>
                <a href="#" class="hover:text-cyan-400 transition-colors">Blog</a>
            </div>
            <button class="px-8 py-3 bg-white text-black font-semibold rounded-2xl hover:bg-cyan-400 hover:text-black transition-all flex items-center gap-2">
                <i class="fa-brands fa-github"></i>
                Login with GitHub
            </button>
        </div>
    </nav>

    <div class="pt-28 max-w-5xl mx-auto px-8">
        <div class="flex items-start justify-between">
            <div>
                <h1 class="text-6xl font-black tracking-tighter text-cyan-400">"#)) (name) (PreEscaped(r#"</h1>
                <p class="text-2xl text-zinc-400 mt-2">v"#)) (latest.version) (PreEscaped(r#"</p>
            </div>
            <div class="text-right">
                <div class="inline-flex items-center gap-2 bg-emerald-500/10 text-emerald-400 px-6 py-3 rounded-3xl text-sm font-medium">
                    <i class="fa-solid fa-download"></i> "#)) (latest.downloads) (PreEscaped(r#" downloads
                </div>
            </div>
        </div>

        <div class="mt-8 bg-zinc-900 border border-zinc-800 rounded-3xl p-10">
            <div class="flex items-center justify-between mb-8">
                <div class="flex items-center gap-4">
                    <button onclick="navigator.clipboard.writeText('zorb add "#)) (name) (PreEscaped(r#"')" 
                            class="px-8 py-4 bg-zinc-800 hover:bg-zinc-700 rounded-2xl flex items-center gap-3 text-lg font-medium">
                        <i class="fa-solid fa-copy"></i>
                        zorb add "#)) (name) (PreEscaped(r#"
                    </button>
                </div>
                <a href="#" class="px-10 py-4 bg-cyan-400 text-black font-bold rounded-2xl flex items-center gap-3 hover:bg-cyan-300 transition">
                    <i class="fa-solid fa-download"></i> Download latest (v"#)) (latest.version) (PreEscaped(r#")
                </a>
            </div>

            <p class="text-xl text-zinc-300 leading-relaxed">"#)) (latest.description.clone().unwrap_or_else(|| "No description provided yet.".to_string())) (PreEscaped(r#"</p>

            <div class="mt-12 grid grid-cols-3 gap-6 text-sm">
                <div>
                    <div class="text-zinc-500">License</div>
                    <div class="font-medium text-white mt-1">"#)) (latest.license.clone().unwrap_or_else(|| "MIT".to_string())) (PreEscaped(r#"</div>
                </div>
                <div>
                    <div class="text-zinc-500">Repository</div>
                    <div class="font-medium text-white mt-1 break-all">"#)) (latest.repository.clone().unwrap_or_else(|| "—".to_string())) (PreEscaped(r#"</div>
                </div>
                <div>
                    <div class="text-zinc-500">Published</div>
                    <div class="font-medium text-white mt-1">"#)) (latest.created_at.format("%b %d, %Y").to_string()) (PreEscaped(r#"</div>
                </div>
            </div>
        </div>

        <!-- Dependencies Section -->
        <div class="mt-16 bg-zinc-900 border border-zinc-800 rounded-3xl p-10">
            <h2 class="text-3xl font-semibold mb-8 flex items-center gap-3">
                <span class="text-cyan-400">🔗</span> Dependencies
            </h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="bg-zinc-950 border border-zinc-700 rounded-2xl p-6">
                    <a href="/@zeta/tokio" class="font-mono text-cyan-400 hover:text-cyan-300">@zeta/tokio ^1.42</a>
                    <p class="text-xs text-zinc-500 mt-1">The async runtime that powers Zeta</p>
                </div>
                <div class="bg-zinc-950 border border-zinc-700 rounded-2xl p-6">
                    <a href="/@http/hyper" class="font-mono text-cyan-400 hover:text-cyan-300">@http/hyper ^1.3</a>
                    <p class="text-xs text-zinc-500 mt-1">Low-level HTTP implementation</p>
                </div>
                <div class="bg-zinc-950 border border-zinc-700 rounded-2xl p-6">
                    <a href="/@data/serde" class="font-mono text-cyan-400 hover:text-cyan-300">@data/serde ^1.0</a>
                    <p class="text-xs text-zinc-500 mt-1">Fast &amp; safe serialization</p>
                </div>
                <div class="bg-zinc-950 border border-zinc-700 rounded-2xl p-6">
                    <a href="/@logging/tracing" class="font-mono text-cyan-400 hover:text-cyan-300">@logging/tracing ^0.2</a>
                    <p class="text-xs text-zinc-500 mt-1">Structured, performant logging</p>
                </div>
            </div>
        </div>

        <!-- Version History -->
        <div class="mt-16">
            <h2 class="text-3xl font-semibold mb-8 flex items-center gap-3">
                <span class="text-cyan-400">📜</span> Version History
            </h2>
            <div class="bg-zinc-900 border border-zinc-800 rounded-3xl overflow-hidden">
                <table class="w-full">
                    <thead class="bg-zinc-950">
                        <tr>
                            <th class="px-8 py-5 text-left text-sm font-medium text-zinc-400">Version</th>
                            <th class="px-8 py-5 text-left text-sm font-medium text-zinc-400">Published</th>
                            <th class="px-8 py-5 text-left text-sm font-medium text-zinc-400">Downloads</th>
                            <th class="px-8 py-5 text-right text-sm font-medium text-zinc-400">Action</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-zinc-800">
"#))
            @for v in &versions {
                html! {
                    tr class="hover:bg-zinc-800 transition" {
                        td class="px-8 py-6 font-mono text-cyan-400" { (v.version) }
                        td class="px-8 py-6 text-zinc-400" { (v.created_at.format("%b %d, %Y").to_string()) }
                        td class="px-8 py-6 text-zinc-400" { (v.downloads) " downloads" }
                        td class="px-8 py-6 text-right" {
                            a href="#" class="text-cyan-400 hover:text-cyan-300 font-medium flex items-center justify-end gap-2" {
                                "Download "
                                i class="fa-solid fa-arrow-down" {}
                            }
                        }
                    }
                }
            }
            (PreEscaped(r#"
                    </tbody>
                </table>
            </div>
        </div>
    </div>

    <footer class="border-t border-zinc-800 py-12 text-center text-zinc-500 text-sm mt-20">
        Powered by The Zeta Foundation • © 2026 zorbs.io
    </footer>
</body>
</html>
            "#))
        }
    }
}

async fn search_zorbs(Query(params): Query<SearchParams>, State(state): State<Arc<AppState>>) -> Markup {
    let search_term = params.q.unwrap_or_default().trim().to_lowercase();

    let zorbs: Vec<Zorb> = if search_term.is_empty() {
        sqlx::query_as("SELECT id, name, version, description, license, repository, downloads, created_at FROM zorbs ORDER BY downloads DESC LIMIT 12")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
    } else {
        sqlx::query_as("SELECT id, name, version, description, license, repository, downloads, created_at FROM zorbs WHERE LOWER(name) LIKE $1 OR LOWER(description) LIKE $1 ORDER BY downloads DESC LIMIT 12")
            .bind(format!("%{}%", search_term))
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
    };

    html! {
        div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6" {
            @for zorb in &zorbs {
                a href=(format!("/{}", zorb.name)) class="block" {
                    div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8" {
                        div class="flex justify-between items-start" {
                            div {
                                span class="font-mono text-cyan-400" { (zorb.name) }
                                p class="text-zinc-400 mt-2 text-sm" { (zorb.description.clone().unwrap_or_else(|| "No description".to_string())) }
                            }
                            span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full" { (zorb.version) }
                        }
                        div class="mt-8 text-xs text-zinc-500 flex gap-6" {
                            span { "↓ " (zorb.downloads) }
                            span { "★ " (zorb.downloads / 100) }
                        }
                    }
                }
            }
            @if zorbs.is_empty() {
                div class="col-span-full text-center text-zinc-400 py-20" {
                    p { "No zorbs found matching your search." }
                    p class="text-sm mt-2" { "Try a different term or publish your first zorb!" }
                }
            }
        }
    }
}

async fn publish_zorb(mut multipart: Multipart, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut name = String::new();
    let mut version = String::new();
    let mut description = None;
    let mut file_bytes = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("name") => name = field.text().await.unwrap(),
            Some("version") => version = field.text().await.unwrap(),
            Some("description") => description = Some(field.text().await.unwrap()),
            Some("file") => {
                file_bytes = Some(field.bytes().await.unwrap());
            }
            _ => {}
        }
    }

    if name.is_empty() || version.is_empty() || file_bytes.is_none() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Missing name, version or file"})));
    }

    let filename = format!("{}-{}.zorb", name.replace('/', "_"), version);
    let upload_path = format!("uploads/{}", filename);
    fs::create_dir_all("uploads").await.unwrap();
    fs::write(&upload_path, file_bytes.unwrap()).await.unwrap();

    let zorb_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO zorbs (id, name, version, description, downloads, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 0, NOW(), NOW())
         ON CONFLICT (name, version) DO UPDATE SET updated_at = NOW()",
        zorb_id,
        name,
        version,
        description
    )
    .execute(&state.db)
    .await
    .unwrap();

    tracing::info!("Zorb published: {} v{}", name, version);

    (StatusCode::CREATED, Json(json!({
        "success": true,
        "id": zorb_id,
        "name": name,
        "version": version,
        "message": "Zorb published successfully!"
    })))
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy", "service": "zorbs-registry"})))
}

async fn list_zorbs(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"zorbs": [], "total": 0})))
}
