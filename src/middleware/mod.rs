use axum::{
    body::Body,
    http::{Method, Request},
    middleware::Next,
    response::Response,
};
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600))
}

pub fn create_trace_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO))
}

pub async fn request_logging(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(request).await;
    let duration = start.elapsed();

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration = ?duration,
        "Request processed"
    );

    response
}