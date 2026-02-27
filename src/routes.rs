// src/routes.rs
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;
use crate::state::AppState;
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(crate::handlers::home::homepage))
        .route("/publish", get(crate::handlers::publish::publish_page))
        .route("/@{scope}/{name}", get(crate::handlers::detail::zorb_detail_scoped))
        .route("/{name}", get(crate::handlers::detail::zorb_detail))
        .route("/@{scope}/{name}/{version}/download", get(crate::handlers::download::download_zorb_scoped))
        .route("/{name}/{version}/download", get(crate::handlers::download::download_zorb))
        .route("/api/health", get(crate::handlers::home::health))
        .route("/api/zorbs", get(crate::handlers::home::list_zorbs))
        .route("/api/zorbs/new", post(crate::handlers::publish::publish_zorb))
        .route("/api/search", get(crate::handlers::home::search_zorbs))
        .route("/api/resolve", get(crate::handlers::resolve::resolve_package))
        .route("/auth/github", get(crate::handlers::auth::github_login))
        .route("/auth/github/callback", get(crate::handlers::auth::github_callback))
        .route("/auth/google", get(crate::handlers::auth::google_login))
        .route("/auth/google/callback", get(crate::handlers::auth::google_callback))
        .route("/auth/twitter", get(crate::handlers::auth::twitter_login))
        .route("/auth/twitter/callback", get(crate::handlers::auth::twitter_callback))
        .route("/auth/logout", get(crate::handlers::auth::logout))
        .route("/admin/seed", get(crate::handlers::home::seed_official))
        // Passkeys
        .route("/auth/passkey/register/start", post(crate::handlers::auth::passkey_register_start))
        .route("/auth/passkey/register/finish", post(crate::handlers::auth::passkey_register_finish))
        .route("/auth/passkey/login/start", post(crate::handlers::auth::passkey_login_start))
        .route("/auth/passkey/login/finish", post(crate::handlers::auth::passkey_login_finish))
}
