// src/main.rs
mod config;
mod state;
mod error;
mod db;
mod models;
mod handlers;
mod views;
mod routes;
mod utils;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

use crate::config::Config;
use crate::routes::create_router;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load();
    let state = state::new();

    let app = create_router(state)
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .expect("Failed to bind to port 3000");

    println!("🚀 Zorbs registry listening on http://localhost:{}", config.port);
    axum::serve(listener, app).await.unwrap();
}
