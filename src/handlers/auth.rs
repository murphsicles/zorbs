// src/handlers/auth.rs
use axum::{extract::{Query, State}, response::Redirect};
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
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use crate::state::AppState;
use crate::db;
use crate::models::user::UserBackend;
use crate::config;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
}

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
    Redirect::temporary(auth_url.to_string())
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

    let github_id = user_info["id"].as_i64().unwrap_or(0);
    let username = user_info["login"].as_str().unwrap_or("unknown").to_string();
    let email = user_info["email"].as_str().map(str::to_string);
    let avatar_url = user_info["avatar_url"].as_str().map(str::to_string);

    let user = match db::find_or_create_user(&state.db, github_id, &username, email, avatar_url).await {
        Ok(u) => u,
        Err(_) => return Redirect::to("/?error=user"),
    };
    let _ = auth_session.login(&user).await;
    Redirect::to("/")
}

pub async fn logout(mut auth_session: AuthSession<UserBackend>) -> Redirect {
    auth_session.logout().await;
    Redirect::to("/")
}
