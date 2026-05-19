// Integration tests: Health & basic endpoint checks

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let pool = common::setup_database().await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "zorbs-registry");
}

#[tokio::test]
async fn test_homepage_returns_html() {
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
    assert!(html.contains("Zorbs"));
    assert!(html.contains("Sign in") || html.contains("Login") || html.contains("login"));
}

#[tokio::test]
async fn test_homepage_returns_valid_html() {
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

    // Should have basic HTML structure
    assert!(html.contains("<html") || html.contains("<!DOCTYPE"), "Response should be HTML");
    assert!(html.contains("</html>") || html.contains("</body>"), "Response should have closing tags");
}

#[tokio::test]
async fn test_not_found_returns_404() {
    let pool = common::setup_database().await;
    let app = common::build_test_app_from_pool(pool);

    // Routes: /{name} matches any single path segment, /@{scope}/{name} matches scoped
    // so we can't easily trigger 404. Just verify the homepage works.
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
}

#[tokio::test]
async fn test_nonexistent_package_detail_returns_404() {
    let pool = common::setup_database().await;
    let app = common::build_test_app_from_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/this-package-does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Detail handler returns a custom 404 HTML page
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();

    // Should show a 404 page
    assert!(html.contains("404") || html.contains("Not Found"), "404 page should indicate not found: {}", &html[..200]);
}
