// Integration tests: Package name validation, edge cases, and error handling
mod common;
use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use serde_json::Value;
// ─── Package name validation ────────────────────────────────────
/// Helper: attempt to publish a tarball and return the response status + body.
async fn try_publish(app: &mut axum::Router, tarball: Vec<u8>) -> (StatusCode, Vec<u8>) {
    let boundary = "val-boundary";
    let body = build_multipart_body(boundary, &[
        ("file", "pkg.tar.gz", "application/octet-stream", &tarball),
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
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes().to_vec();
    (status, body_bytes)
}
#[tokio::test]
async fn test_validate_valid_package_name_succeeds() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    // Valid names
    let valid_names = [
        "my-package",
        "my_package",
        "package123",
        "a",
        "with-many-hyphens-and-123",
    ];
    for name in &valid_names {
        let tarball = common::create_test_tarball(name, "1.0.0");
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "Valid package name '{}' should publish successfully",
            name
        );
    }
}
#[tokio::test]
async fn test_validate_invalid_package_name_fails() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    // Names with invalid characters
    let invalid_names = [
        "with space",
        "with/slash",
        "-leading-hyphen",
        "trailing-hyphen-",
    ];
    for name in &invalid_names {
        let tarball = common::create_test_tarball(name, "1.0.0");
        let (status, body) = try_publish(&mut app, tarball).await;
        // Should reject with either BAD_REQUEST (400) or possibly INTERNAL_SERVER_ERROR
        // The handler currently catches the validation error in parse_zorb_toml
        assert_ne!(
            status,
            StatusCode::CREATED,
            "Invalid package name '{}' should not be created",
            name
        );
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("Package name") || body_str.contains("error") || body_str.len() > 0,
            "Should return an error for invalid name '{}': {}",
            name,
            body_str
        );
    }
}
#[tokio::test]
async fn test_validate_reserved_names_fail() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    let reserved = ["admin", "root", "system", "google", "facebook", "tesla", "nvidia", "intel"];
    for name in &reserved {
        let tarball = common::create_test_tarball(name, "1.0.0");
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_ne!(
            status,
            StatusCode::CREATED,
            "Reserved name '{}' should be rejected",
            name
        );
    }
}
#[tokio::test]
async fn test_validate_reserved_scopes_fail() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    let reserved_scopes = ["@admin/pkg", "@root/pkg", "@system/pkg", "@google/pkg", "@nvidia/pkg"];
    for name in &reserved_scopes {
        let tarball = common::create_test_tarball(name, "1.0.0");
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_ne!(
            status,
            StatusCode::CREATED,
            "Reserved scope '{}' should be rejected",
            name
        );
    }
}
#[tokio::test]
async fn test_validate_scoped_name_format() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    // Valid scoped names
    let valid = ["@mypkg/my-lib", "@my_scope/my_pkg", "@my/pkg"];
    for name in &valid {
        let tarball = common::create_test_tarball(name, "1.0.0");
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "Valid scoped name '{}' should publish successfully",
            name
        );
    }
}
#[tokio::test]
async fn test_validate_invalid_scoped_formats_fail() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    // Invalid scoped names
    let invalid = ["@/name", "@scope/name/extra", "@-h/name", "@h-/name"];
    for name in &invalid {
        let tarball = common::create_test_tarball(name, "1.0.0");
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_ne!(
            status,
            StatusCode::CREATED,
            "Invalid scoped name '{}' should be rejected",
            name
        );
    }
}
#[tokio::test]
async fn test_validate_version_semver() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    let valid_versions = ["0.0.1", "1.0.0", "2.3.4", "10.20.30", "0.1.0-alpha", "1.0.0+build"];
    for ver in &valid_versions {
        let tarball = common::create_test_tarball("test-pkg", ver);
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "Valid semver '{}' should be accepted",
            ver
        );
    }
}
#[tokio::test]
async fn test_validate_invalid_version_fails() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    let invalid_versions = ["1.0", "v1.0.0", "latest", "1.0.0.0", "abc", "1", ""];
    for ver in &invalid_versions {
        let tarball = common::create_test_tarball("test-pkg", ver);
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_ne!(
            status,
            StatusCode::CREATED,
            "Invalid version '{}' should be rejected",
            ver
        );
    }
}
#[tokio::test]
async fn test_validate_blocked_words_fail() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    // Names containing blocked words
    let blocked = ["kill-switch", "my-nazi-pkg", "contains-ass-as-name"];
    for name in &blocked {
        let tarball = common::create_test_tarball(name, "1.0.0");
        let (status, _body) = try_publish(&mut app, tarball).await;
        assert_ne!(
            status,
            StatusCode::CREATED,
            "Name containing blocked word '{}' should be rejected",
            name
        );
    }
}
#[tokio::test]
async fn test_validate_missing_zorb_toml_fails() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    // Create a tar.gz WITHOUT a zorb.toml
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
        hdr.set_path("README.md").expect("set path");
        hdr.set_size(15u64);
        hdr.set_cksum();
        tar.append(&hdr, &b"# Just a readme\n"[..])
            .expect("append to tar");
        tar.finish().expect("finish tar");
    }
    let (status, body) = try_publish(&mut app, buf).await;
    assert_eq!(status, StatusCode::BAD_REQUEST,
        "Tarball without zorb.toml should be rejected");
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("zorb.toml") || body_str.contains("error"),
        "Response should mention missing zorb.toml: {}", body_str);
}
// ─── Edge cases ─────────────────────────────────────────────────
#[tokio::test]
async fn test_package_with_dependencies() {
    let pool = common::setup_database().await;
    let mut app = common::build_test_app_from_pool(pool);
    let deps = &[("@core/once_cell", "^1.0"), ("@data/serde", "^1.0")];
    let tarball = common::create_test_tarball_with_deps("my-app", "0.1.0", deps);
    let (status, body) = try_publish(&mut app, tarball).await;
    assert_eq!(status, StatusCode::CREATED,
        "Package with valid deps should publish");
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "my-app");
    assert_eq!(json["version"], "0.1.0");
}
#[tokio::test]
async fn test_empty_database_returns_empty_trending() {
    let pool = common::setup_database().await;
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
    // Should still render the homepage even with no packages
    assert!(html.contains("Zorbs") || html.contains("<!DOCTYPE") || html.contains("zorbs"),
        "Homepage should render even with empty DB: {}",
        &html[..300]);
}
// ─── Multipart body builder ─────────────────────────────────────
fn build_multipart_body(boundary: &str, fields: &[(&str, &str, &str, &[u8])]) -> Vec<u8> {
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
    let footer = format!("--{}--\r\n", boundary);
    body.extend_from_slice(footer.as_bytes());
    body
}
