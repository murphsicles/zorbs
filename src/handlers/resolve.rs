// src/handlers/resolve.rs
use axum::{extract::{Query, State}, Json, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use crate::state::AppState;
use crate::db::queries;

#[derive(Deserialize)]
pub struct ResolveParams {
    pub name: String,
}

pub async fn resolve_package(
    Query(params): Query<ResolveParams>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if let Some(zorb) = queries::get_latest_zorb(&state.db, &params.name).await {
        let download_url = if params.name.starts_with('@') {
            format!("/{}{}/{}", params.name, if params.name.contains('/') { "" } else { "/" }, zorb.version)
        } else {
            format!("/{}/{}", params.name, zorb.version)
        } + "/download";

        Json(json!({
            "name": zorb.name,
            "version": zorb.version,
            "download_url": download_url
        }))
    } else {
        Json(json!({"error": "Package not found"}))
    }
}
