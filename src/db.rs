use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!()
        .run(pool)
        .await
        .expect("Failed to run database migrations");
}

// Future query helpers go here
pub mod queries {
    use super::*;
    use crate::models::Zorb;
    use sqlx::Row;

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
}
