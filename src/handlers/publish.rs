use axum::{Json, extract::{State, Multipart}, response::IntoResponse, http::StatusCode};
use serde_json::json;
use maud::{html, Markup, PreEscaped};
use std::sync::Arc;
use tokio::fs;

use crate::state::AppState;
use crate::models::NewZorb;
use crate::utils;
use crate::views::publish::PUBLISH_HTML;

pub async fn publish_page() -> Markup {
    html! { (PreEscaped(PUBLISH_HTML)) }
}

pub async fn publish_zorb(mut multipart: Multipart, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut new_zorb = NewZorb {
        name: String::new(),
        version: String::new(),
        description: None,
        license: None,
        repository: None,
    };
    let mut file_bytes = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("name") => new_zorb.name = field.text().await.unwrap(),
            Some("version") => new_zorb.version = field.text().await.unwrap(),
            Some("description") => new_zorb.description = Some(field.text().await.unwrap()),
            Some("license") => new_zorb.license = Some(field.text().await.unwrap()),
            Some("repository") => new_zorb.repository = Some(field.text().await.unwrap()),
            Some("file") => file_bytes = Some(field.bytes().await.unwrap()),
            _ => {}
        }
    }

    if new_zorb.name.is_empty() || new_zorb.version.is_empty() || file_bytes.is_none() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Missing required fields"})));
    }

    let filename = utils::zorb_filename(&new_zorb.name, &new_zorb.version);
    let upload_path = format!("{}/{}", crate::config::upload_dir(), filename);
    fs::create_dir_all(&crate::config::upload_dir()).await.unwrap();
    fs::write(&upload_path, file_bytes.unwrap()).await.unwrap();

    let id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO zorbs (id, name, version, description, license, repository, downloads, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 0, NOW(), NOW())
         ON CONFLICT (name, version) DO UPDATE SET updated_at = NOW()",
        id,
        new_zorb.name,
        new_zorb.version,
        new_zorb.description,
        new_zorb.license,
        new_zorb.repository
    )
    .execute(&state.db)
    .await
    .unwrap();

    (StatusCode::CREATED, Json(json!({
        "success": true,
        "id": id,
        "name": new_zorb.name,
        "version": new_zorb.version,
        "message": "Zorb published successfully!"
    })))
}
