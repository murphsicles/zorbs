// src/state.rs
use sqlx::PgPool;
use std::sync::Arc;
use crate::config;
use crate::models::user::UserBackend;  // NEW

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub backend: UserBackend,  // NEW (clone-safe for layers)
}

pub fn new() -> Arc<AppState> {
    let db = PgPool::connect_lazy(&config::database_url())
        .expect("Failed to create DB pool");
    let backend = UserBackend::new(db.clone());  // NEW
    Arc::new(AppState { db, backend })
}
