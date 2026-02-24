// src/main.rs
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use tower_sessions::{SessionManagerLayer, Expiry, MemoryStore};
use axum_login::AuthManagerLayerBuilder;
use time::Duration;

mod config;
mod state;
mod error;
mod db;
mod models;
mod handlers;
mod views;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = state::new();
    db::run_migrations(&state.db).await;

    let session_store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // dev/localhost
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let auth_layer = AuthManagerLayerBuilder::new(state.backend.clone(), session_layer).build();

    let app = Router::new()
        .merge(routes::routes())
        .layer(auth_layer)
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(config::addr())
        .await
        .expect("Failed to bind");
    tracing::info!("🚀 Zorbs registry listening on {}", config::addr());
    axum::serve(listener, app).await.unwrap();
}
