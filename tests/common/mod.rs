// tests/common/mod.rs — shared test helpers for Zenith registry integration tests

use sqlx::{PgPool, Connection};
use sqlx::postgres::{PgConnection, PgPoolOptions};

const TEST_DB: &str = "zorbs_test";

/// Each call sets up a fresh test database with migrations, and returns a pool.
/// The DB is created only once (idempotent check), but each test gets a clean pool
/// to avoid connection contention from shared state.
pub async fn setup_database() -> PgPool {
    // First-time: ensure the test database exists
    let admin_url = admin_database_url();
    let mut admin_conn = PgConnection::connect(&admin_url)
        .await
        .expect("Failed to connect to admin DB");

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)"
    )
    .bind(TEST_DB)
    .fetch_one(&mut admin_conn)
    .await
    .unwrap_or(false);

    if !exists {
        let sql = format!("CREATE DATABASE {}", TEST_DB);
        sqlx::query(&sql).execute(&mut admin_conn).await.unwrap();
    }
    admin_conn.close().await;

    // Connect with a fresh pool for this test
    let test_url = test_database_url();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&test_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations (idempotent) and clean tables
    zorbs::db::run_migrations(&pool).await;
    clean_tables(&pool).await;
    pool
}

async fn clean_tables(pool: &PgPool) {
    let mut conn = pool.acquire().await.expect("acquire for cleanup");
    for table in &["webauthn_credentials", "sessions", "zorbs", "users"] {
        let _ = sqlx::query(&format!("DELETE FROM {}", table))
            .execute(&mut *conn)
            .await;
    }
}

/// Build a test app from a pool.
pub fn build_test_app_from_pool(pool: PgPool) -> axum::Router {
    let backend = zorbs::models::user::UserBackend::new(pool.clone());

    let rp_origin = url::Url::parse("http://localhost:9999")
        .expect("test rp_origin");
    let webauthn = std::sync::Arc::new(
        webauthn_rs::prelude::WebauthnBuilder::new("localhost", &rp_origin)
            .expect("webauthn config")
            .build()
            .expect("webauthn build")
    );

    let app_state = std::sync::Arc::new(zorbs::state::AppState {
        db: pool,
        backend,
        webauthn,
    });

    zorbs::build_app(app_state)
}

// ─── Helpers ────────────────────────────────────────────────────

fn admin_database_url() -> String {
    let base = zorbs::config::database_url();
    let slash = base.rfind('/').expect("DATABASE_URL should contain /");
    format!("{}/postgres", &base[..slash])
}

fn test_database_url() -> String {
    let base = zorbs::config::database_url();
    let slash = base.rfind('/').expect("DATABASE_URL should contain /");
    format!("{}/{}", &base[..slash], TEST_DB)
}

/// Build a single-entry tar.gz archive.
fn build_tar_entry(path: &str, content: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let gz = flate2::write::GzEncoder::new(&mut buf, flate2::Compression::fast());
        let mut tar = tar::Builder::new(gz);
        let mut hdr = tar::Header::new_ustar();
        hdr.set_entry_type(tar::EntryType::Regular);
        hdr.set_mode(0o644);
        hdr.set_mtime(0);
        hdr.set_uid(0);
        hdr.set_gid(0);
        hdr.set_path(path).expect("set tar path");
        hdr.set_size(content.len() as u64);
        hdr.set_cksum();
        tar.append(&hdr, content).expect("append tar entry");
        tar.finish().expect("finish tar");
    }
    buf
}

/// Create a test tarball with a zorb.toml.
pub fn create_test_tarball(name: &str, version: &str) -> Vec<u8> {
    let toml = format!(
        r#"[package]
name = "{}"
version = "{}"
description = "Test package for integration testing"
license = "MIT"
"#, name, version
    );
    build_tar_entry("zorb.toml", toml.as_bytes())
}

/// Create a test tarball with dependencies.
pub fn create_test_tarball_with_deps(name: &str, version: &str, deps: &[(&str, &str)]) -> Vec<u8> {
    let deps_str = deps.iter()
        .map(|(k, v)| format!(r#""{}" = "{}""#, k, v))
        .collect::<Vec<_>>()
        .join("\n");
    let toml = format!(
        r#"[package]
name = "{}"
version = "{}"
description = "Test package with deps"
license = "MIT"

[dependencies]
{}
"#, name, version, deps_str
    );
    build_tar_entry("zorb.toml", toml.as_bytes())
}
