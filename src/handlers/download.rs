// src/handlers/download.rs
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::fs;
use crate::{state::AppState, config, db::queries};

async fn serve_file(name: String, version: String, state: Arc<AppState>) -> impl IntoResponse {
    let filename = crate::utils::zorb_filename(&name, &version); // reuse your existing helper
    let path = format!("{}/{}", config::upload_dir(), filename);

    if !fs::try_exists(&path).await.unwrap_or(false) {
        return (StatusCode::NOT_FOUND, "Zorb not found").into_response();
    }

    // Increment download count
    let _ = sqlx::query!(
        "UPDATE zorbs SET downloads = downloads + 1 WHERE name = $1 AND version = $2",
        name, version
    )
    .execute(&state.db)
    .await;

    match fs::read(&path).await {
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
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
    }
}

// Flat name (if we ever support them)
pub async fn download_zorb(
    Path((name, version)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    serve_file(name, version, state).await
}

// Scoped name (@scope/name)
pub async fn download_zorb_scoped(
    Path((scope, name, version)): Path<(String, String, String)>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let full_name = format!("@{}/{}", scope, name);
    serve_file(full_name, version, state).await
}
