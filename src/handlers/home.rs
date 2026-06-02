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

    // Fetch aggregate stats for hero pills
    let (total_packages, total_downloads) = queries::get_home_stats(&state.db).await;

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

    // Inject stats pills into hero
    let stats_pills = format!(
        r##"<div class="flex justify-center gap-3 mt-4">
            <div class="text-xs bg-emerald-500/10 text-emerald-400 px-4 py-1.5 rounded-full whitespace-nowrap font-medium">
                📦 <span class="countup" data-target="{pkgs}">0</span> packages
            </div>
            <div class="text-xs bg-cyan-500/10 text-cyan-400 px-4 py-1.5 rounded-full whitespace-nowrap font-medium">
                ⬇ <span class="countup" data-target="{dls}">0</span> downloads
            </div>
        </div>"##,
        pkgs = total_packages,
        dls = total_downloads,
    );
    html_str = html_str.replace("<!-- STATS_PILLS -->", &stats_pills);

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

pub async fn list_zorbs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    use crate::models::Zorb;
    match sqlx::query_as::<_, Zorb>(
        "SELECT id, name, version, description, license, repository, owner_id, downloads, created_at, updated_at, dependencies, readme FROM zorbs ORDER BY downloads DESC"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(zorbs) => {
            let total = zorbs.len();
            let zorbs_json: Vec<serde_json::Value> = zorbs.into_iter().map(|z| {
                json!({
                    "id": z.id,
                    "name": z.name,
                    "version": z.version,
                    "description": z.description,
                    "license": z.license,
                    "repository": z.repository,
                    "downloads": z.downloads,
                    "owner_id": z.owner_id
                })
            }).collect();
            (StatusCode::OK, Json(json!({"zorbs": zorbs_json, "total": total})))
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("DB query failed: {}", e), "zorbs": [], "total": 0})))
        }
    }
}

fn generate_minimal_zorb(name: &str, version: &str, description: &str, license: &str, repository: &Option<String>) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut tar_builder = tar::Builder::new(flate2::write::GzEncoder::new(&mut buf, flate2::Compression::default()));

    // zorb.toml
    let toml_content = format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2026\"\ndescription = \"{}\"\nlicense = \"{}\"{}\n\n",
        name,
        version,
        description,
        license,
        repository.as_ref().map(|r| format!("\nrepository = \"{}\"", r)).unwrap_or_default()
    );
    let mut header = tar::Header::new_gnu();
    header.set_path("zorb.toml").unwrap();
    header.set_size(toml_content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar_builder.append(&header, std::io::Cursor::new(toml_content.as_bytes())).unwrap();

    // Placeholder src/mod.z
    let src_content = format!(
        "// {} v{}\n// {}\n\npub fn version() -> &'static str {{\n    \"{}\"\n}}\n",
        name, version, description, version
    );
    let mut header = tar::Header::new_gnu();
    header.set_path("src/mod.z").unwrap();
    header.set_size(src_content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar_builder.append(&header, std::io::Cursor::new(src_content.as_bytes())).unwrap();

    // Finalize gzip
    let _ = tar_builder.finish().unwrap();
    drop(tar_builder);
    buf
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

        // Also generate and store a minimal .zorb file so downloads work out of the box
        let filename = crate::utils::zorb_filename(name, version);
        let repo_opt: Option<String> = repository.map(|s| s.to_string());
        let zorb_bytes = generate_minimal_zorb(name, version, description, license, &repo_opt);
        let _ = state.storage.store(&filename, &zorb_bytes).await;
    }
    Redirect::to("/")
}
