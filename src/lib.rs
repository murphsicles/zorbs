// lib.rs — Zorbs registry library (crate root for all source modules)

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod state;
pub mod utils;
pub mod views;
pub mod storage;

use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tower_sessions::{SessionManagerLayer, Expiry, MemoryStore};
use axum_login::AuthManagerLayerBuilder;
use time::Duration;

/// Build the full Axum application for a given AppState.
/// Shared between the server binary and integration tests.
pub fn build_app(state: Arc<state::AppState>) -> Router {
    let session_store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let auth_layer = AuthManagerLayerBuilder::new(state.backend.clone(), session_layer).build();

    Router::new()
        .merge(routes::routes())
        .layer(auth_layer)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
