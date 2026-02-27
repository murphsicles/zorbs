// src/handlers/auth.rs
use axum::{extract::{Query, State}, response::Redirect, Json};
use axum_login::AuthSession;
use oauth2::{
    basic::BasicClient,
    AuthUrl,
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl,
};
use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use crate::state::AppState;
use crate::db;
use crate::models::user::UserBackend;
use crate::config;
use webauthn_rs::prelude::*;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
}

// === Existing OAuth handlers (unchanged) ===
pub async fn github_login() -> Redirect { /* unchanged */ }
pub async fn github_callback(...) -> Redirect { /* unchanged */ }
pub async fn google_login() -> Redirect { /* unchanged */ }
pub async fn google_callback(...) -> Redirect { /* unchanged */ }
pub async fn twitter_login() -> Redirect { /* unchanged */ }
pub async fn twitter_callback(...) -> Redirect { /* unchanged */ }
pub async fn logout(mut auth_session: AuthSession<UserBackend>) -> Redirect { /* unchanged */ }

// === Passkey (WebAuthn 2.0) handlers ===
#[derive(Deserialize)]
pub struct PasskeyRegisterStart {
    username: String,
}

#[derive(Serialize)]
pub struct PasskeyRegisterStartResponse {
    pub public_key_credential_creation_options: CreationChallengeResponse,
}

pub async fn passkey_register_start(
    Json(payload): Json<PasskeyRegisterStart>,
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<UserBackend>,
) -> Json<PasskeyRegisterStartResponse> {
    let user_id = Uuid::new_v4();
    let (ccr, skr) = state.webauthn
        .start_passkey_registration(user_id, &payload.username, &payload.username, None)
        .expect("Failed to start registration");

    // Store registration state in session (for finish)
    let _ = auth_session.insert("webauthn_reg_state", skr).await;

    Json(PasskeyRegisterStartResponse { public_key_credential_creation_options: ccr })
}

#[derive(Deserialize)]
pub struct PasskeyRegisterFinish {
    username: String,
    response: RegisterPublicKeyCredential,
}

pub async fn passkey_register_finish(
    Json(payload): Json<PasskeyRegisterFinish>,
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<UserBackend>,
) -> Redirect {
    let skr: PasskeyRegistration = match auth_session.get("webauthn_reg_state").await {
        Ok(Some(s)) => s,
        _ => return Redirect::to("/?error=reg_state"),
    };

    let _ = auth_session.remove("webauthn_reg_state").await;

    let reg = match state.webauthn.finish_passkey_registration(&payload.response, &skr) {
        Ok(r) => r,
        Err(_) => return Redirect::to("/?error=reg_finish"),
    };

    // Create user + save credential (TODO: store credential in DB if you want multiple)
    let user = match db::find_or_create_user(&state.db, "passkey", &reg.credential_id().to_string(), &payload.username, None, None).await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/?error=user"),
    };

    let _ = auth_session.login(&user).await;
    Redirect::to("/")
}

#[derive(Deserialize)]
pub struct PasskeyLoginStart {
    username: String,
}

#[derive(Serialize)]
pub struct PasskeyLoginStartResponse {
    pub public_key_credential_request_options: RequestChallengeResponse,
}

pub async fn passkey_login_start(
    Json(payload): Json<PasskeyLoginStart>,
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<UserBackend>,
) -> Json<PasskeyLoginStartResponse> {
    // TODO: lookup credential IDs for this username from DB
    let allowed_credentials = vec![]; // populate from DB in production

    let (rcr, skr) = state.webauthn
        .start_passkey_authentication(allowed_credentials)
        .expect("Failed to start login");

    let _ = auth_session.insert("webauthn_login_state", skr).await;

    Json(PasskeyLoginStartResponse { public_key_credential_request_options: rcr })
}

#[derive(Deserialize)]
pub struct PasskeyLoginFinish {
    response: PublicKeyCredential,
}

pub async fn passkey_login_finish(
    Json(payload): Json<PasskeyLoginFinish>,
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<UserBackend>,
) -> Redirect {
    let skr: PasskeyAuthentication = match auth_session.get("webauthn_login_state").await {
        Ok(Some(s)) => s,
        _ => return Redirect::to("/?error=login_state"),
    };

    let _ = auth_session.remove("webauthn_login_state").await;

    let _ = match state.webauthn.finish_passkey_authentication(&payload.response, &skr) {
        Ok(_) => (),
        Err(_) => return Redirect::to("/?error=login_finish"),
    };

    // TODO: lookup user by credential_id from DB and login
    // For now redirect to home (extend with real lookup)
    Redirect::to("/")
}
