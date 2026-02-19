// src/db/queries.rs
use sqlx::PgPool;
use crate::models::Zorb;

pub async fn list_zorbs(pool: &PgPool) -> Vec<Zorb> {
    sqlx::query_as("SELECT id, name, version, description, license, repository, downloads, created_at FROM zorbs ORDER BY downloads DESC LIMIT 12")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}

pub async fn search_zorbs(pool: &PgPool, term: &str) -> Vec<Zorb> {
    let term = format!("%{}%", term.to_lowercase());
    sqlx::query_as("SELECT id, name, version, description, license, repository, downloads, created_at FROM zorbs WHERE LOWER(name) LIKE $1 OR LOWER(description) LIKE $1 ORDER BY downloads DESC LIMIT 12")
        .bind(term)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}

pub async fn get_zorb_versions(pool: &PgPool, name: &str) -> Vec<Zorb> {
    sqlx::query_as("SELECT id, name, version, description, license, repository, downloads, created_at FROM zorbs WHERE name = $1 ORDER BY created_at DESC")
        .bind(name)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}

pub async fn get_latest_zorb(pool: &PgPool, name: &str) -> Option<Zorb> {
    sqlx::query_as("SELECT id, name, version, description, license, repository, downloads, created_at FROM zorbs WHERE name = $1 ORDER BY created_at DESC LIMIT 1")
        .bind(name)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
}
