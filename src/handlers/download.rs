// src/handlers/download.rs
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::fs;
use crate::state::AppState;
use crate::config;
use crate::utils;

async fn serve_file(name: String, version: String, state: Arc<AppState>) -> impl IntoResponse {
    let filename = utils::zorb_filename(&name, &version);
    let upload_path = format!("{}/{}", config::upload_dir(), filename);
    if !fs::try_exists(&upload_path).await.unwrap_or(false) {
        return (StatusCode::NOT_FOUND, "Zorb not found").into_response();
    }
        let _ = sqlx::query!(
        "UPDATE zorbs SET downloads = downloads + 1 WHERE name = $1 AND version = $2",
        name,
        version
    )
    .execute(&state.db)
    .await;
    match fs::read(&upload_path).await {
        Ok(bytes) => {
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename)
                    .parse()
                    .unwrap(),
            );
            (headers, bytes).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read zorb file").into_response(),
    }
}

pub async fn download_zorb(
    Path((name, version)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    serve_file(name, version, state).await
}

pub async fn download_zorb_scoped(
    Path((scope, name, version)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let full_name = format!("@{}/{}", scope, name);
    serve_file(full_name, version, state).await
}
