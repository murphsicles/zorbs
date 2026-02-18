use axum::{
    routing::{get, post},
    Router, Json, response::IntoResponse,
    http::StatusCode,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Beautiful logging from day one
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // Public routes
        .route("/", get(root))
        .route("/api/health", get(health))
        
        // Future Zorbs API
        .route("/api/crates", get(list_crates))
        .route("/api/crates/new", post(publish_zorb))
        
        // Middleware
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("🚀 Zorbs registry listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

// Welcome page
async fn root() -> impl IntoResponse {
    Json(json!({
        "message": "Welcome to zorbs.io — the official Zeta package registry",
        "docs": "https://docs.zorbs.io",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Health check (used by Docker, Kubernetes, etc.)
async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy", "service": "zorbs-registry"})))
}

// Placeholder endpoints — we’ll fill these next
async fn list_crates() -> impl IntoResponse {
    // TODO: query Postgres, return paginated list
    (StatusCode::OK, Json(json!({"crates": []})))
}

async fn publish_zorb() -> impl IntoResponse {
    // TODO: trusted publishing, security scan, store tarball
    (StatusCode::OK, Json(json!({"success": true})))
}
