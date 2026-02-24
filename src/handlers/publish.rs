// src/handlers/publish.rs
use axum::{Json, extract::{State, Multipart}, response::IntoResponse, http::StatusCode};
use serde_json::json;
use maud::{html, Markup, PreEscaped};
use axum_login::AuthSession;
use std::sync::Arc;
use tokio::fs;
use crate::state::AppState;
use crate::models::NewZorb;
use crate::utils;
use crate::config;
use crate::views;
use crate::models::user::UserBackend;

pub async fn publish_page(auth_session: AuthSession<UserBackend>) -> Markup {
    let mut html_str = views::PUBLISH_HTML.to_string();

    // dynamic nav (exact copy from homepage)
    let user = &auth_session.user;
    let nav_html = if let Some(user) = user {
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
            a href="/auth/github" class="px-8 py-3 bg-white text-black font-semibold rounded-2xl hover:bg-cyan-400 hover:text-black transition-all flex items-center gap-2" {
                i class="fa-brands fa-github" {}
                "Login with GitHub"
            }
        }
    };
    if let Some(pos) = html_str.find(r#"<a href="/auth/github" class="px-8 py-3 bg-white text-black font-semibold rounded-2xl hover:bg-cyan-400 hover:text-black transition-all flex items-center gap-2">"#) {
        let end = html_str[pos..].find("</a>").map(|i| pos + i + 4).unwrap_or(html_str.len());
        html_str.replace_range(pos..end, &nav_html.into_string());
    }

    html! { (PreEscaped(html_str)) }
}

pub async fn publish_zorb(State(state): State<Arc<AppState>>, mut multipart: Multipart) -> impl IntoResponse {
    let mut form_name = String::new();
    let mut form_version = String::new();
    let mut form_description: Option<String> = None;
    let mut form_license: Option<String> = None;
    let mut form_repository: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name() {
            Some("name") => form_name = field.text().await.unwrap_or_default(),
            Some("version") => form_version = field.text().await.unwrap_or_default(),
            Some("description") => form_description = Some(field.text().await.unwrap_or_default()),
            Some("license") => form_license = Some(field.text().await.unwrap_or_default()),
            Some("repository") => form_repository = Some(field.text().await.unwrap_or_default()),
            Some("file") => file_bytes = field.bytes().await.ok().map(|b| b.to_vec()),
            _ => {}
        }
    }
    let file_bytes_vec = match file_bytes {
        Some(bytes) if !bytes.is_empty() => bytes,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({"error": "File upload is required"}))),
    };
    let new_zorb = match utils::parse_zorb_toml(&file_bytes_vec) {
        Ok(parsed) => parsed,
        Err(err) => {
            if form_name.is_empty() || form_version.is_empty() {
                return (StatusCode::BAD_REQUEST, Json(json!({"error": err})));
            }
            NewZorb {
                name: form_name,
                version: form_version,
                description: form_description,
                license: form_license,
                repository: form_repository,
            }
        }
    };
    let filename = utils::zorb_filename(&new_zorb.name, &new_zorb.version);
    let upload_path = format!("{}/{}", config::upload_dir(), filename);
    fs::create_dir_all(&config::upload_dir()).await.unwrap();
    fs::write(&upload_path, &file_bytes_vec).await.unwrap();
    let id = uuid::Uuid::new_v4();
    let _ = sqlx::query!(
        "INSERT INTO zorbs (id, name, version, description, license, repository, downloads, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 0, NOW(), NOW())
         ON CONFLICT (name, version) DO UPDATE SET
            description = EXCLUDED.description,
            license = EXCLUDED.license,
            repository = EXCLUDED.repository,
            updated_at = NOW()",
        id,
        new_zorb.name,
        new_zorb.version,
        new_zorb.description,
        new_zorb.license,
        new_zorb.repository
    )
    .execute(&state.db)
    .await;
    (StatusCode::CREATED, Json(json!({
        "success": true,
        "id": id,
        "name": new_zorb.name,
        "version": new_zorb.version,
        "message": "Zorb published successfully! Metadata validated and extracted from zorb.toml."
    })))
}
