// src/db.rs
use sqlx::PgPool;
use crate::models::User;

pub async fn find_or_create_user(
    pool: &PgPool,
    provider: &str,
    provider_id: &str,
    username: &str,
    email: Option<String>,
    avatar_url: Option<String>,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (provider, provider_id, username, email, avatar_url)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (provider, provider_id) DO UPDATE SET
            username = EXCLUDED.username,
            email = EXCLUDED.email,
            avatar_url = EXCLUDED.avatar_url,
            updated_at = NOW()
        RETURNING *
        "#,
        provider,
        provider_id,
        username,
        email,
        avatar_url
    )
    .fetch_one(pool)
    .await
}

pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!()
        .run(pool)
        .await
        .expect("Failed to run database migrations");
}

pub mod queries {
    use super::*;
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
}
