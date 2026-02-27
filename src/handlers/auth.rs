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
use crate::models::user::{User, UserBackend};
use crate::config;
use webauthn_rs::prelude::*;
use uuid::Uuid;
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use serde_cbor_2;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
}
// === OAuth handlers ===
pub async fn github_login() -> Redirect {
    let redirect = format!("{}/auth/github/callback", config::registry_url());
    let client = BasicClient::new(ClientId::new(config::github_client_id()))
        .set_client_secret(ClientSecret::new(config::github_client_secret()))
        .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect).unwrap());
    let (auth_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".into()))
        .add_scope(Scope::new("user:email".into()))
        .url();
    Redirect::temporary(auth_url.as_str())
}
pub async fn github_callback(
    Query(query): Query<CallbackQuery>,
    mut auth_session: AuthSession<UserBackend>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let redirect = format!("{}/auth/github/callback", config::registry_url());
    let client = BasicClient::new(ClientId::new(config::github_client_id()))
        .set_client_secret(ClientSecret::new(config::github_client_secret()))
        .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect).unwrap());
    let http_client = ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .expect("reqwest client");
    let token = match client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/?error=token"),
    };
    let http = HttpClient::new();
    let user_info: Value = match http
        .get("https://api.github.com/user")
        .header("User-Agent", "zorbs-registry")
        .bearer_auth(token.access_token().secret())
        .send()
        .await {
        Ok(r) => match r.json().await {
            Ok(u) => u,
            Err(_) => return Redirect::to("/?error=profile"),
        },
        Err(_) => return Redirect::to("/?error=profile"),
    };
    let provider_id = user_info["id"].as_i64().unwrap_or(0).to_string();
    let username = user_info["login"].as_str().unwrap_or("unknown").to_string();
    let email = user_info["email"].as_str().map(str::to_string);
    let avatar_url = user_info["avatar_url"].as_str().map(str::to_string);
    let user = match db::find_or_create_user(&state.db, "github", &provider_id, &username, email, avatar_url).await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/?error=user"),
    };
    let _ = auth_session.login(&user).await;
    Redirect::to("/")
}
pub async fn google_login() -> Redirect {
    let redirect = format!("{}/auth/google/callback", config::registry_url());
    let client = BasicClient::new(ClientId::new(config::google_client_id()))
        .set_client_secret(ClientSecret::new(config::google_client_secret()))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect).unwrap());
    let (auth_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("profile".into()))
        .add_scope(Scope::new("email".into()))
        .url();
    Redirect::temporary(auth_url.as_str())
}
pub async fn google_callback(
    Query(query): Query<CallbackQuery>,
    mut auth_session: AuthSession<UserBackend>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let redirect = format!("{}/auth/google/callback", config::registry_url());
    let client = BasicClient::new(ClientId::new(config::google_client_id()))
        .set_client_secret(ClientSecret::new(config::google_client_secret()))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect).unwrap());
    let http_client = ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .expect("reqwest client");
    let token = match client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/?error=token"),
    };
    let http = HttpClient::new();
    let user_info: Value = match http
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .header("User-Agent", "zorbs-registry")
        .bearer_auth(token.access_token().secret())
        .send()
        .await {
        Ok(r) => match r.json().await {
            Ok(u) => u,
            Err(_) => return Redirect::to("/?error=profile"),
        },
        Err(_) => return Redirect::to("/?error=profile"),
    };
    let provider_id = user_info["id"].as_str().unwrap_or("0").to_string();
    let username = user_info["name"].as_str().unwrap_or("unknown").to_string();
    let email = user_info["email"].as_str().map(str::to_string);
    let avatar_url = user_info["picture"].as_str().map(str::to_string);
    let user = match db::find_or_create_user(&state.db, "google", &provider_id, &username, email, avatar_url).await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/?error=user"),
    };
    let _ = auth_session.login(&user).await;
    Redirect::to("/")
}
pub async fn twitter_login() -> Redirect {
    let redirect = format!("{}/auth/twitter/callback", config::registry_url());
    let client = BasicClient::new(ClientId::new(config::twitter_client_id()))
        .set_client_secret(ClientSecret::new(config::twitter_client_secret()))
        .set_auth_uri(AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect).unwrap());
    let (auth_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("users.read".into()))
        .add_scope(Scope::new("tweet.read".into()))
        .url();
    Redirect::temporary(auth_url.as_str())
}
pub async fn twitter_callback(
    Query(query): Query<CallbackQuery>,
    mut auth_session: AuthSession<UserBackend>,
    State(state): State<Arc<AppState>>,
) -> Redirect {
    let redirect = format!("{}/auth/twitter/callback", config::registry_url());
    let client = BasicClient::new(ClientId::new(config::twitter_client_id()))
        .set_client_secret(ClientSecret::new(config::twitter_client_secret()))
        .set_auth_uri(AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string()).unwrap())
        .set_token_uri(TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string()).unwrap())
        .set_redirect_uri(RedirectUrl::new(redirect).unwrap());
    let http_client = ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .expect("reqwest client");
    let token = match client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/?error=token"),
    };
    let http = HttpClient::new();
    let user_info: Value = match http
        .get("https://api.twitter.com/2/users/me")
        .header("User-Agent", "zorbs-registry")
        .bearer_auth(token.access_token().secret())
        .send()
        .await {
        Ok(r) => match r.json().await {
            Ok(u) => u,
            Err(_) => return Redirect::to("/?error=profile"),
        },
        Err(_) => return Redirect::to("/?error=profile"),
    };
    let provider_id = user_info["data"]["id"].as_str().unwrap_or("0").to_string();
    let username = user_info["data"]["username"].as_str().unwrap_or("unknown").to_string();
    let email = None;
    let avatar_url = user_info["data"]["profile_image_url"].as_str().map(str::to_string);
    let user = match db::find_or_create_user(&state.db, "twitter", &provider_id, &username, email, avatar_url).await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/?error=user"),
    };
    let _ = auth_session.login(&user).await;
    Redirect::to("/")
}
pub async fn logout(mut auth_session: AuthSession<UserBackend>) -> Redirect {
    let _ = auth_session.logout().await;
    Redirect::to("/")
}
// === Passkey handlers ===
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
    let skr_json = serde_json::to_string(&skr).unwrap();
    let _ = auth_session.session.insert("webauthn_reg_state", skr_json).await;
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
    let skr_json: Option<String> = auth_session.session.get("webauthn_reg_state").await.unwrap_or(None);
    let skr: PasskeyRegistration = serde_json::from_str(&skr_json.unwrap_or_default()).unwrap_or_else(|_| panic!("reg_state"));
    let _ = auth_session.session.remove::<String>("webauthn_reg_state").await;
    let reg = match state.webauthn.finish_passkey_registration(&payload.response, &skr) {
        Ok(r) => r,
        Err(_) => return Redirect::to("/?error=reg_finish"),
    };
    let cred_id_str = STANDARD_NO_PAD.encode(reg.cred_id().as_ref());
    let user = match db::find_or_create_user(&state.db, "passkey", &cred_id_str, &payload.username, None, None).await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/?error=user"),
    };
    let public_key_bytes = serde_cbor_2::to_vec(reg.get_public_key()).unwrap();
    let _ = sqlx::query!(
        "INSERT INTO webauthn_credentials (user_id, credential_id, public_key, counter)
         VALUES ($1, $2, $3, $4)",
        user.id,
        cred_id_str,
        public_key_bytes,
        0i64
    ).execute(&state.db).await;
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
    Json(_payload): Json<PasskeyLoginStart>,
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<UserBackend>,
) -> Json<PasskeyLoginStartResponse> {
    let (rcr, skr) = state.webauthn
        .start_passkey_authentication(&[])
        .expect("Failed to start login");
    let skr_json = serde_json::to_string(&skr).unwrap();
    let _ = auth_session.session.insert("webauthn_login_state", skr_json).await;
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
    let skr_json: Option<String> = auth_session.session.get("webauthn_login_state").await.unwrap_or(None);
    let skr: PasskeyAuthentication = serde_json::from_str(&skr_json.unwrap_or_default()).unwrap_or_else(|_| panic!("login_state"));
    let _ = auth_session.session.remove::<String>("webauthn_login_state").await;
    let auth_result = match state.webauthn.finish_passkey_authentication(&payload.response, &skr) {
        Ok(r) => r,
        Err(_) => return Redirect::to("/?error=login_finish"),
    };
    let cred_id_str = STANDARD_NO_PAD.encode(auth_result.cred_id().as_ref());
    let user: Option<User> = sqlx::query_as!(
        User,
        "SELECT id, username, email, provider, provider_id, avatar_url, created_at, updated_at FROM users WHERE id = (SELECT user_id FROM webauthn_credentials WHERE credential_id = $1)",
        cred_id_str
    ).fetch_optional(&state.db).await.unwrap_or(None);
    let user = match user {
        Some(u) => u,
        None => return Redirect::to("/?error=user"),
    };
    let _ = auth_session.login(&user).await;
    Redirect::to("/")
}
