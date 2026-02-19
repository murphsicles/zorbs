// src/routes.rs
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;
use crate::state::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(crate::handlers::home::homepage))
        .route("/publish", get(crate::handlers::publish::publish_page))
        .route("/:name", get(crate::handlers::detail::zorb_detail))
        .route("/@:scope/:name", get(crate::handlers::detail::zorb_detail_scoped))
        .route("/api/health", get(crate::handlers::home::health))
        .route("/api/zorbs", get(crate::handlers::home::list_zorbs))
        .route("/api/zorbs/new", post(crate::handlers::publish::publish_zorb))
        .route("/api/search", get(crate::handlers::home::search_zorbs))
}
