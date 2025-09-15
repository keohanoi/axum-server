use axum::{routing::get, Router};
use axum::http::{StatusCode, Request};
use axum::body;
use tower::ServiceExt; // for `oneshot`

#[tokio::test]
async fn health_endpoint_returns_ok() {
    async fn health_check() -> &'static str { "OK" }
    let app = Router::new().route("/health", get(health_check));

    let response = app
        .oneshot(Request::get("/health").body(String::new()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(&body[..], b"OK");
}
