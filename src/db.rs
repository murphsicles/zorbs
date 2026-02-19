use sqlx::PgPool;
use crate::models::{Zorb, NewZorb};
use uuid::Uuid;
use chrono::Utc;

pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!().run(pool).await.expect("Failed to run migrations");
}

pub async fn create_zorb(pool: &PgPool, new_zorb: NewZorb) -> Result<Zorb, sqlx::Error> {
    let id = Uuid::new_v4();
    let created_at = Utc::now();
    sqlx::query_as!(
        Zorb,
        r#"
        INSERT INTO zorbs (id, name, version, description, license, repository, downloads, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, version, description, license, repository, downloads, created_at
        "#,
        id,
        new_zorb.name,
        new_zorb.version,
        new_zorb.description,
        new_zorb.license,
        new_zorb.repository,
        0i64,
        created_at
    )
    .fetch_one(pool)
    .await
}

pub async fn get_zorb(pool: &PgPool, name: &str, version: &str) -> Option<Zorb> {
    sqlx::query_as!(
        Zorb,
        r#"
        SELECT id, name, version, description, license, repository, downloads, created_at
        FROM zorbs
        WHERE name = $1 AND version = $2
        "#,
        name,
        version
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None)
}

pub async fn list_trending_zorbs(pool: &PgPool) -> Vec<Zorb> {
    sqlx::query_as!(
        Zorb,
        r#"
        SELECT id, name, version, description, license, repository, downloads, created_at
        FROM zorbs
        ORDER BY downloads DESC
        LIMIT 12
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

pub async fn search_zorbs(pool: &PgPool, term: &str) -> Vec<Zorb> {
    let term = format!("%{}%", term.to_lowercase());
    sqlx::query_as!(
        Zorb,
        r#"
        SELECT id, name, version, description, license, repository, downloads, created_at
        FROM zorbs
        WHERE LOWER(name) LIKE $1 OR LOWER(description) LIKE $1
        ORDER BY downloads DESC
        LIMIT 12
        "#,
        term
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}
