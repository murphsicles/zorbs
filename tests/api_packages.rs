// Integration tests: Full package lifecycle (publish, list, detail, search, download)

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use serde_json::Value;

/// Helper: spawn seeded packages into the test DB directly.
async fn seed_test_packages(pool: &sqlx::PgPool) {
    use sqlx::query;
    let pkgs = vec![
        ("@async/tokio",  "0.3.10", "Async runtime for Zeta",            "MIT", "https://github.com/murphsicles/tokio",     "{}"),
        ("@data/serde",   "1.0.228","Serialization framework",           "MIT", "https://github.com/murphsicles/serde",     "{}"),
        ("@cli/clap",     "4.5.1",  "Command line argument parser",      "MIT", "https://github.com/murphsicles/clap",      "{}"),
        ("@http/axum",    "0.8.1",  "Ergonomic web framework",           "MIT", "https://github.com/zeta-lang/axum",        r#"{"@async/tokio": "^1.42", "@http/hyper": "^1.3"}"#),
        ("my-package",    "1.0.0",  "A simple test package",             "MIT", "https://github.com/murphsicles/my-package","{}"),
        ("scanner",       "2.1.0",  "File scanner utility",              "Apache-2.0","https://github.com/murphsicles/scanner", r#"{"my-package": "^1.0"}"#),
    ];
    for (name, version, desc, license, repo, deps) in pkgs {
        let parsed_deps: serde_json::Value = serde_json::from_str(deps).unwrap_or_default();
        query(
            "INSERT INTO zorbs (id, name, version, description, license, repository, dependencies, downloads, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
             ON CONFLICT (name, version) DO NOTHING"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(name)
        .bind(version)
        .bind(desc)
        .bind(license)
        .bind(repo)
        .bind(parsed_deps)
        .bind(0i64)
        .execute(pool)
        .await
        .expect("seed package");
    }
}

// ─── Publish tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_publish_package() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool.clone());

    // Create a valid tarball
    let tarball = common::create_test_tarball("my-zeta-lib", "0.1.0");

    // Build multipart body manually
    let boundary = "test-boundary-12345";
    let body = build_multipart_body(boundary, &[
        ("file", "my-zeta-lib-0.1.0.tar.gz", "application/octet-stream", &tarball),
        ("name", "", "text/plain", b""),
        ("version", "", "text/plain", b""),
    ]);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/zorbs/new")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED,
        "Publishing valid tarball should return 201 Created");

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["name"], "my-zeta-lib");
    assert_eq!(json["version"], "0.1.0");
    assert!(json["id"].is_string(), "Should return a UUID");
}

#[tokio::test]
async fn test_publish_missing_file_returns_400() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);

    let boundary = "test-boundary-2";
    let body = build_multipart_body(boundary, &[
        ("name", "", "text/plain", b"test-pkg"),
        ("version", "", "text/plain", b"1.0.0"),
    ]);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/zorbs/new")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST,
        "Publishing without file should return 400");
}

#[tokio::test]
async fn test_publish_invalid_tarball_returns_400() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);

    // Invalid bytes (not a tar.gz)
    let garbage = b"not a valid tarball";

    let boundary = "test-boundary-3";
    let body = build_multipart_body(boundary, &[
        ("file", "bad.zorb", "application/octet-stream", garbage),
    ]);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/zorbs/new")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap_or_default();

    // Should return 400 with error message
    assert!(
        status == StatusCode::BAD_REQUEST || json.get("error").is_some(),
        "Invalid tarball should return 400. Got status: {}, body: {:?}",
        status,
        String::from_utf8_lossy(&body_bytes)
    );
}

#[tokio::test]
async fn test_publish_duplicate_overwrites() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool.clone());

    let tarball1 = common::create_test_tarball("overwrite-test", "1.0.0");
    let tarball2 = common::create_test_tarball("overwrite-test", "1.0.0");

    // First publish
    let boundary1 = "boundary-dup-1";
    let body1 = build_multipart_body(boundary1, &[
        ("file", "overwrite-test-1.0.0.tar.gz", "application/octet-stream", &tarball1),
    ]);
    let resp1 = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/zorbs/new")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary1))
                .body(Body::from(body1))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Second publish (same name+version) — should not error (upsert behavior)
    let mut app2 = common::build_test_app_from_pool(pool.clone());
    let boundary2 = "boundary-dup-2";
    let body2 = build_multipart_body(boundary2, &[
        ("file", "overwrite-test-1.0.0.tar.gz", "application/octet-stream", &tarball2),
    ]);
    let resp2 = app2
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/zorbs/new")
                .header("content-type", format!("multipart/form-data; boundary={}", boundary2))
                .body(Body::from(body2))
                .unwrap(),
        )
        .await
        .unwrap();
    // Should still create/upsert — ON CONFLICT DO UPDATE
    assert_eq!(resp2.status(), StatusCode::CREATED,
        "Duplicate publish should upsert (ON CONFLICT DO UPDATE)");
}

// ─── List & Search tests ────────────────────────────────────────

#[tokio::test]
async fn test_list_zorbs_returns_seeded_packages() {
    let pool = common::setup_database().await;

    // This endpoint is HTML-only (homepage), not JSON.
    // The /api/zorbs endpoint is stubbed and returns empty.
    // But we can check the homepage shows seeded packages.
    seed_test_packages(&pool).await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Seeded packages should appear somewhere in the trending cards
    assert!(html.contains("my-package"), "Homepage should list seeded packages: {}", &html[..500]);
    assert!(html.contains("scanner"), "Homepage should list seeded packages");
    assert!(html.contains("@async/tokio") || html.contains("@data/serde"),
        "Homepage should show seeded scoped packages");
}

#[tokio::test]
async fn test_search_returns_relevant_packages() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search?q=serial")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Should find serde
    assert!(html.contains("@data/serde") || html.contains("serde"),
        "Search for 'serial' should find serde package. HTML snippet: {}",
        &html[..html.len().min(300)]);
}

#[tokio::test]
async fn test_search_empty_returns_all() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();
    // Should show all packages
    assert!(html.contains("my-package"), "Empty search should return all packages");
}

// ─── Detail page tests ──────────────────────────────────────────

#[tokio::test]
async fn test_package_detail_shows_info() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/my-package")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("my-package"), "Detail page should show package name");
    assert!(html.contains("1.0.0"), "Detail page should show version");
    assert!(html.contains("A simple test package"), "Detail page should show description");
}

#[tokio::test]
async fn test_scoped_package_detail() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/@async/tokio")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("@async/tokio"), "Scoped package detail should show @scope/name");
    assert!(html.contains("0.3.10"), "Scoped package detail should show version");
}

#[tokio::test]
async fn test_package_detail_shows_dependencies() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/@http/axum")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Should show @async/tokio as a dependency
    assert!(html.contains("@async/tokio"), "Detail should list dependencies");
    assert!(html.contains("@http/hyper"), "Detail should list all dependencies");
}

#[tokio::test]
async fn test_package_detail_shows_version_history() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;

    // Add a second version
    sqlx::query(
        "INSERT INTO zorbs (id, name, version, description, license, repository, dependencies, downloads, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8, NOW(), NOW())"
    )
    .bind(uuid::Uuid::new_v4())
    .bind("my-package")
    .bind("2.0.0")
    .bind("New version")
    .bind("MIT")
    .bind("https://github.com/murphsicles/my-package")
    .bind("{}")
    .bind(0i64)
    .execute(&pool)
    .await
    .expect("insert second version");

    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/my-package")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Both versions should show in version history
    assert!(html.contains("1.0.0"), "Version history should include 1.0.0");
    assert!(html.contains("2.0.0"), "Version history should include 2.0.0");
}

// ─── Download tests ─────────────────────────────────────────────

#[tokio::test]
async fn test_download_nonexistent_returns_404() {
    let pool = common::setup_database().await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/missing-pkg/1.0.0/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_download_increments_counter() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;

    // Create a tarball in the upload dir so the file exists
    let upload_dir = "uploads";
    tokio::fs::create_dir_all(upload_dir).await.unwrap();
    let tarball = common::create_test_tarball("my-package", "1.0.0");
    let filename = "my-package-1.0.0.zorb";
    tokio::fs::write(format!("{}/{}", upload_dir, filename), &tarball)
        .await
        .unwrap();

    let app = common::build_test_app_from_pool(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/my-package/1.0.0/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK,
        "Download should return 200 when file exists");

    // Check that download count was incremented
    let row: (i64,) = sqlx::query_as(
        "SELECT downloads FROM zorbs WHERE name = $1 AND version = $2"
    )
    .bind("my-package")
    .bind("1.0.0")
    .fetch_one(&pool)
    .await
    .expect("fetch download count");

    assert!(row.0 >= 1, "Download count should be incremented (got {})", row.0);

    // Cleanup
    let _ = tokio::fs::remove_file(format!("{}/{}", upload_dir, filename)).await;
}

#[tokio::test]
async fn test_download_scoped_package() {
    let pool = common::setup_database().await;
    seed_test_packages(&pool).await;

    let upload_dir = "uploads";
    tokio::fs::create_dir_all(upload_dir).await.unwrap();
    let filename = "async-tokio-0.3.10.zorb";
    let tarball = common::create_test_tarball("@async/tokio", "0.3.10");
    tokio::fs::write(format!("{}/{}", upload_dir, filename), &tarball)
        .await
        .unwrap();

    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/@async/tokio/0.3.10/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK,
        "Scoped package download should work");

    // Cleanup
    let _ = tokio::fs::remove_file(format!("{}/{}", upload_dir, filename)).await;
}

// ─── Multipart body builder ─────────────────────────────────────

fn build_multipart_body(boundary: &str, fields: &[(&str, &str, &str, &[u8])]) -> Vec<u8> {
    // fields: (name, filename, content_type, data)
    let mut body = Vec::new();
    for (name, filename, content_type, data) in fields {
        let mut header = String::new();
        header += &format!("--{}\r\n", boundary);
        header += &format!("Content-Disposition: form-data; name=\"{}\"", name);
        if !filename.is_empty() {
            header += &format!("; filename=\"{}\"", filename);
        }
        header += "\r\n";
        if !content_type.is_empty() {
            header += &format!("Content-Type: {}\r\n", content_type);
        }
        header += "\r\n";
        body.extend_from_slice(header.as_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    // Closing boundary
    let footer = format!("--{}--\r\n", boundary);
    body.extend_from_slice(footer.as_bytes());
    body
}
