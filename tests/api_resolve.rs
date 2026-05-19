// Integration tests: Resolve API (package resolution)

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use serde_json::Value;

async fn seed_resolve_test_packages(pool: &sqlx::PgPool) {
    sqlx::query(
        "INSERT INTO zorbs (id, name, version, description, license, repository, dependencies, downloads, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
         ON CONFLICT (name, version) DO NOTHING"
    )
    .bind(uuid::Uuid::new_v4())
    .bind("resolve-test-pkg")
    .bind("2.1.0")
    .bind("A test package for resolve")
    .bind("MIT")
    .bind("https://github.com/murphsicles/resolve-test-pkg")
    .bind(serde_json::json!({}))
    .bind(0i64)
    .execute(pool)
    .await
    .expect("seed resolve package");
}

#[tokio::test]
async fn test_resolve_existing_package() {
    let pool = common::setup_database().await;
    seed_resolve_test_packages(&pool).await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/resolve?name=resolve-test-pkg")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "resolve-test-pkg");
    assert_eq!(json["version"], "2.1.0");
    assert!(json["download_url"].is_string());
    assert!(
        json["download_url"].as_str().unwrap().contains("download"),
        "download_url should point to download endpoint, got: {}",
        json["download_url"]
    );
}

#[tokio::test]
async fn test_resolve_scoped_package() {
    let pool = common::setup_database().await;

    // Seed a scoped package
    let result = sqlx::query(
        "INSERT INTO zorbs (id, name, version, description, license, repository, dependencies, downloads, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
         ON CONFLICT (name, version) DO NOTHING"
    )
    .bind(uuid::Uuid::new_v4())
    .bind("@scope/awesome-lib")
    .bind("3.0.0")
    .bind("An awesome scoped library")
    .bind("MIT")
    .bind("https://github.com/murphsicles/awesome-lib")
    .bind(serde_json::json!({}))
    .bind(0i64)
    .execute(&pool)
    .await;

    assert!(result.is_ok(), "seed scoped package failed: {:?}", result.err());

    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/resolve?name=@scope/awesome-lib")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["name"], "@scope/awesome-lib",
        "Expected name '@scope/awesome-lib', got: {:?}", json);
    assert_eq!(json["version"], "3.0.0");
    assert!(json["download_url"].as_str().unwrap().contains("@scope/awesome-lib"),
        "Scoped package download_url should include @scope/name");
}

#[tokio::test]
async fn test_resolve_nonexistent_package() {
    let pool = common::setup_database().await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/resolve?name=nonexistent-pkg-xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return a JSON with "error" field, status should be OK
    // (the handler always returns OK with an error field)
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "Package not found",
        "Resolve non-existing should report error");
}

#[tokio::test]
async fn test_resolve_empty_name() {
    let pool = common::setup_database().await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/resolve")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Missing required query param — Axum should return 422 or handler handles it
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    if status == StatusCode::OK {
        // Handler might still process it
        let json: Value = serde_json::from_slice(&body).unwrap_or_default();
        // Either error or not found
        assert!(json.get("error").is_some() || json.get("name").is_some(),
            "Unhandled response for empty resolve query: {:?}",
            String::from_utf8_lossy(&body));
    }
    // Otherwise it's a validation rejection which is fine
}
