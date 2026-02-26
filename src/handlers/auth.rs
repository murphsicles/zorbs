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
    let email = None;  // X doesn't provide email
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
