use axum_server::{config::Config, db, kafka::EventProducer, routes};
use axum::{http::{StatusCode, Request}};
use axum::body;
use tower::ServiceExt; // for oneshot

// Note: This test requires a running Postgres matching DATABASE_URL.
// Kafka is optional; set KAFKA_ENABLED=false for determinism.
#[tokio::test]
async fn get_health_ok_on_full_app() {
    dotenvy::dotenv().ok();

    // Build minimal config and app
    let cfg = Config::from_env().expect("load config");

    // Try DB pool; if not permitted or unavailable, skip test gracefully.
    let pool = match db::create_pool(&cfg.database_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("skipping integration test: cannot connect to DB: {e}");
            return; // treat as skipped in constrained environments
        }
    };
    // Migrations (no-op if already applied)
    let _ = db::run_migrations(&pool).await;

    let producer = match EventProducer::new(cfg.kafka.clone()).await {
        Ok(p) => p,
        Err(_) => {
            let mut disabled = cfg.kafka.clone();
            disabled.enabled = false;
            EventProducer::new(disabled).await.expect("disabled producer")
        }
    };

    let app = routes::create_routes(pool, producer);
    let response = app
        .oneshot(Request::get("/health").body(String::new()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), 1024).await.unwrap();
    assert_eq!(&body[..], b"OK");
}
