// src/state.rs
use sqlx::PgPool;
use std::sync::Arc;
use crate::config;
use crate::models::user::UserBackend;
use webauthn_rs::prelude::*;
use url::Url;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub backend: UserBackend,
    pub webauthn: Arc<Webauthn>, // NEW for Passkeys
}

pub fn new() -> Arc<AppState> {
    let db = PgPool::connect_lazy(&config::database_url())
        .expect("Failed to create DB pool");
    let backend = UserBackend::new(db.clone());

    let rp_origin = Url::parse(&config::webauthn_rp_origin())
        .expect("Invalid WEBAUTHN_RP_ORIGIN in .env");
    let webauthn = Arc::new(
        WebauthnBuilder::new(&config::webauthn_rp_id(), &rp_origin)
            .expect("Invalid Webauthn config")
            .build()
            .expect("Failed to build Webauthn")
    );

    Arc::new(AppState { db, backend, webauthn })
}
