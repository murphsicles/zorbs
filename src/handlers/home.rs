// src/handlers/home.rs
use axum::{Json, extract::{State, Query}, response::IntoResponse, http::StatusCode};
use serde_json::json;
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use std::sync::Arc;
use axum_login::AuthSession;
use crate::state::AppState;
use crate::db::queries;
use crate::views;
use crate::models::User;

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

pub async fn homepage(auth_session: AuthSession<User>) -> Markup {
    // TODO (next step): make nav dynamic with user / logout link
    html! { (PreEscaped(views::HOME_HTML)) }
}

pub async fn search_zorbs(Query(params): Query<SearchParams>, State(state): State<Arc<AppState>>) -> Markup {
    let term = params.q.unwrap_or_default().trim().to_lowercase();
    let zorbs = if term.is_empty() {
        queries::list_zorbs(&state.db).await
    } else {
        queries::search_zorbs(&state.db, &term).await
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
        }
    }
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy", "service": "zorbs-registry"})))
}

pub async fn list_zorbs(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"zorbs": [], "total": 0})))
}

pub async fn seed_official(State(state): State<Arc<AppState>>) -> Redirect {
    let official = vec![
        ("@data/serde", "1.0.210", "Fast & safe serialization", "MIT OR Apache-2.0", Some("https://github.com/zeta-lang/serde")),
        ("@async/tokio", "1.42.0", "The async runtime that powers Zeta", "MIT", Some("https://github.com/zeta-lang/tokio")),
        ("@http/axum", "0.8.1", "Ergonomic web framework", "MIT", Some("https://github.com/zeta-lang/axum")),
        ("@core/once_cell", "1.19.0", "Single assignment cells", "MIT OR Apache-2.0", Some("https://github.com/zeta-lang/once_cell")),
        ("@log/tracing", "0.2.5", "Structured, performant logging", "MIT", Some("https://github.com/zeta-lang/tracing")),
        ("@cli/clap", "4.5.0", "Command line argument parser", "MIT OR Apache-2.0", Some("https://github.com/zeta-lang/clap")),
    ];

    for (name, version, description, license, repository) in official {
        let _ = sqlx::query!(
            "INSERT INTO zorbs (id, name, version, description, license, repository, downloads, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, 0, NOW(), NOW())
             ON CONFLICT (name, version) DO NOTHING",
            uuid::Uuid::new_v4(),
            name,
            version,
            Some(description),
            Some(license),
            repository
        )
        .execute(&state.db)
        .await;
    }

    Redirect::to("/")
}
