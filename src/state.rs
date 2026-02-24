// src/state.rs
use sqlx::PgPool;
use std::sync::Arc;
use tower_sessions::SessionManagerLayer;
use tower_sessions::memory_store::MemoryStore;
use crate::config;
use time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn new() -> (Arc<AppState>, SessionManagerLayer<MemoryStore>) {
    let db = PgPool::connect_lazy(&config::database_url())
        .expect("Failed to create DB pool");

    let session_store = MemoryStore::new();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // true in prod
        .with_http_only(true)
        .with_same_site(tower_sessions::cookie::SameSite::Strict)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::minutes(30)));

    (Arc::new(AppState { db }), session_layer)
}
