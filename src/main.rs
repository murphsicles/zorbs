use axum::{
    routing::{get, post},
    Router, Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/api/health", get(health))
        .route("/api/zorbs", get(list_zorbs))
        .route("/api/zorbs/new", post(publish_zorb))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("🚀 Zorbs registry v{} listening on http://localhost:3000", env!("CARGO_PKG_VERSION"));
    tracing::info!("   The official package registry for Zeta — powered by The Zeta Foundation");

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    Json(json!({
        "service": "zorbs.io",
        "message": "Welcome to the official Zeta package registry",
        "version": env!("CARGO_PKG_VERSION"),
        "docs": "https://docs.zorbs.io",
        "foundation": "The Zeta Foundation"
    }))
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "status": "healthy",
        "service": "zorbs-registry"
    })))
}

// Core placeholders — we'll make these real next
async fn list_zorbs() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"zorbs": [], "total": 0})))
}

async fn publish_zorb() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"message": "Zorb published successfully (stub)"})))
}
