// src/config.rs
use std::env;

pub fn addr() -> String {
    env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
}

pub fn database_url() -> String {
    env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env")
}

pub fn upload_dir() -> String {
    env::var("UPLOAD_DIR")
        .unwrap_or_else(|_| "uploads".to_string())
}

pub fn registry_url() -> String {
    env::var("REGISTRY_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
}

pub fn github_client_id() -> String {
    env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set")
}

pub fn github_client_secret() -> String {
    env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set")
}

pub fn google_client_id() -> String {
    env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set")
}

pub fn google_client_secret() -> String {
    env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set")
}

pub fn twitter_client_id() -> String {
    env::var("TWITTER_CLIENT_ID").expect("TWITTER_CLIENT_ID must be set")
}

pub fn twitter_client_secret() -> String {
    env::var("TWITTER_CLIENT_SECRET").expect("TWITTER_CLIENT_SECRET must be set")
}
