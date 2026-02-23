// src/main.rs
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber;
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
    let (state, session_layer) = state::new();
    db::run_migrations(&state.db).await;
    let app = Router::new()
        .merge(routes::routes())
        .layer(session_layer)
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(config::addr())
        .await
        .expect("Failed to bind");
    tracing::info!("🚀 Zorbs registry listening on {}", config::addr());
    axum::serve(listener, app).await.unwrap();
}
