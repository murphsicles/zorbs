// src/error.rs
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use serde_json::json;
use axum::Json;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}
