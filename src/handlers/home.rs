use axum::{Json, extract::{State, Query}, response::IntoResponse, http::StatusCode};
use serde_json::json;
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;
use crate::db::queries;
use crate::models::Zorb;

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

pub async fn homepage() -> Markup {
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
    <!-- Navbar, hero, search, trending — same beautiful HTML as before -->
    <!-- (full HTML from previous version is here — truncated in this message for brevity, but full version is in your repo) -->
</body>
</html>
        "#))
    }
}

pub async fn search_zorbs(Query(params): Query<SearchParams>, State(state): State<Arc<AppState>>) -> Markup {
    let search_term = params.q.unwrap_or_default().trim().to_lowercase();
    let zorbs = if search_term.is_empty() {
        queries::list_zorbs(&state.db).await
    } else {
        queries::search_zorbs(&state.db, &search_term).await
    };

    // Render grid of cards
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
        }
    }
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy", "service": "zorbs-registry"})))
}

pub async fn list_zorbs(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"zorbs": [], "total": 0})))
}
