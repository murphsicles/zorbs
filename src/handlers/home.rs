// src/handlers/home.rs
use axum::{Json, extract::{State, Query}, response::IntoResponse, http::StatusCode};
use axum::response::Redirect;
use serde_json::json;
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use std::sync::Arc;
use axum_login::AuthSession;
use crate::state::AppState;
use crate::db::queries;
use crate::views;
use crate::models::user::UserBackend;

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
}

pub async fn homepage(auth_session: AuthSession<UserBackend>, State(state): State<Arc<AppState>>) -> Markup {
    let user = &auth_session.user;
    let auth_markup = if let Some(user) = user {
        html! {
            div class="flex items-center gap-6" {
                span class="text-sm font-medium text-zinc-300" { "@" (user.username) }
                a href="/auth/logout" class="px-6 py-3 bg-red-500/10 hover:bg-red-500/20 text-red-400 font-medium rounded-2xl transition-all" {
                    "Logout"
                }
            }
        }
    } else {
        html! {
            button onclick="openLoginModal()" class="px-8 py-3 bg-white text-black font-semibold rounded-2xl hover:bg-cyan-400 hover:text-black transition-all flex items-center gap-2" {
                "Sign in"
                i class="fa-solid fa-right-to-bracket" {}
            }
        }
    };
    let mut html_str = views::HOME_HTML.to_string();
    let auth_str = auth_markup.into_string();
    if let Some(pos) = html_str.find("<!-- AUTH_SLOT -->") {
        html_str.replace_range(pos..pos + "<!-- AUTH_SLOT -->".len(), &auth_str);
    }
    if let Some(pos) = html_str.find("<!-- AUTH_SLOT_MOBILE -->") {
        html_str.replace_range(pos..pos + "<!-- AUTH_SLOT_MOBILE -->".len(), &auth_str);
    }

    // Build dynamic trending cards from top downloaded zorbs
    let trending = queries::list_zorbs(&state.db).await;
    let trending_cards: String = trending.iter().map(|zorb| {
        let href = format!("/{}", zorb.name);
        let downloads_str = if zorb.downloads >= 1_000_000 {
            format!("{}M", zorb.downloads / 1_000_000)
        } else if zorb.downloads >= 1_000 {
            format!("{}k", zorb.downloads / 1_000)
        } else {
            zorb.downloads.to_string()
        };
        let stars = zorb.downloads / 100;
        let stars_str = if stars >= 1_000 {
            format!("{}.{}k", stars / 1_000, (stars % 1_000) / 100)
        } else {
            stars.to_string()
        };
        format!(
            r##"<a href="{href}" class="block h-full">
                <div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8 h-full flex flex-col">
                    <div class="flex-1 flex justify-between items-start">
                        <div>
                            <span class="font-mono text-cyan-400">{name}</span>
                            <p class="text-zinc-400 mt-2 text-sm">{desc}</p>
                        </div>
                        <span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full whitespace-nowrap">v{version}</span>
                    </div>
                    <div class="mt-8 text-xs text-zinc-500 flex gap-6">
                        <span>↓ {downloads}</span>
                        <span>★ {stars}</span>
                    </div>
                </div>
            </a>"##,
            href = href,
            name = zorb.name,
            desc = zorb.description.as_deref().unwrap_or("No description"),
            version = zorb.version,
            downloads = downloads_str,
            stars = stars_str,
        )
    }).collect::<Vec<_>>().join("\n");
    html_str = html_str.replace("<!-- TRENDING_CARDS -->", &trending_cards);

    html! { (PreEscaped(html_str)) }
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
                a href=(format!("/{}", zorb.name)) class="block h-full" {
                    div class="zorb-card bg-zinc-900 border border-zinc-800 rounded-3xl p-8 h-full flex flex-col" {
                        div class="flex-1 flex justify-between items-start" {
                            div {
                                span class="font-mono text-cyan-400" { (zorb.name) }
                                p class="text-zinc-400 mt-2 text-sm" { (zorb.description.clone().unwrap_or_else(|| "No description".to_string())) }
                            }
                            span class="text-xs bg-emerald-500/10 text-emerald-400 px-3 py-1 rounded-full whitespace-nowrap" { (zorb.version) }
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
    use serde_json::json;
    let official = vec![
        ("@data/serde", "0.4.0", "Serialization/Deserialization framework for Zeta", "MIT", Some("https://github.com/murphsicles/serde"), json!({})),
        ("@async/tokio", "1.42.0", "The async runtime that powers Zeta", "MIT", Some("https://github.com/zeta-lang/tokio"), json!({})),
        ("@http/axum", "0.8.1", "Ergonomic web framework", "MIT", Some("https://github.com/zeta-lang/axum"), json!({"@async/tokio": "^1.42", "@http/hyper": "^1.3"})),
        ("@core/once_cell", "1.19.0", "Single assignment cells", "MIT OR Apache-2.0", Some("https://github.com/zeta-lang/once_cell"), json!({})),
        ("@log/tracing", "0.2.5", "Structured, performant logging", "MIT", Some("https://github.com/zeta-lang/tracing"), json!({"@core/once_cell": "^1.19"})),
        ("@cli/clap", "4.5.0", "Command line argument parser", "MIT OR Apache-2.0", Some("https://github.com/zeta-lang/clap"), json!({})),
    ];
    for (name, version, description, license, repository, deps) in official {
        let _ = sqlx::query!(
            "INSERT INTO zorbs (id, name, version, description, license, repository, downloads, created_at, updated_at, dependencies)
             VALUES ($1, $2, $3, $4, $5, $6, 0, NOW(), NOW(), $7)
             ON CONFLICT (name, version) DO NOTHING",
            uuid::Uuid::new_v4(),
            name,
            version,
            Some(description),
            Some(license),
            repository,
            deps as _,
        )
        .execute(&state.db)
        .await;
    }
    Redirect::to("/")
}
