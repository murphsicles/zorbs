// src/handlers/download.rs
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
};
use std::sync::Arc;
use crate::state::AppState;
use crate::utils;

/// Redirect to the package download URL (S3/R2 or local path).
/// Increments the download counter.
async fn serve_file(name: String, version: String, state: Arc<AppState>) -> impl IntoResponse {
    let filename = utils::zorb_filename(&name, &version);

    // Increment download counter
    let _ = sqlx::query!(
        "UPDATE zorbs SET downloads = downloads + 1 WHERE name = $1 AND version = $2",
        name,
        version
    )
    .execute(&state.db)
    .await;

    let url = state.storage.download_url(&filename);

    // If storage backend is S3/R2, redirect to the public URL.
    // If local, proxy through nginx (local URL).
    // For local storage the URL is relative (nginx-served), use a redirect.
    if url.starts_with('/') {
        // Local storage — redirect to nginx-served path
        Redirect::to(&url).into_response()
    } else {
        // S3/R2 — redirect to the public URL
        let headers = axum::http::HeaderMap::from_iter([(
            header::LOCATION,
            url.parse().unwrap(),
        )]);
        (StatusCode::FOUND, headers).into_response()
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
