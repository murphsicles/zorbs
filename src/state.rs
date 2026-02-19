// src/state.rs
use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn new() -> Arc<AppState> {
    let db = PgPool::connect_lazy(&config::database_url())
        .expect("Failed to create DB pool");

    Arc::new(AppState { db })
}

impl FromRef<Arc<AppState>> for PgPool {
    fn from_ref(state: &Arc<AppState>) -> PgPool {
        state.db.clone()
    }
}
