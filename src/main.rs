use axum_server::{config::Config, db, routes};
use std::process;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    let config = Config::from_env().unwrap_or_else(|err| {
        eprintln!("Failed to load configuration: {}", err);
        process::exit(1);
    });

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.rust_log)),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = db::create_pool(&config.database_url)
        .await
        .unwrap_or_else(|err| {
            tracing::error!("Failed to create database pool: {}", err);
            process::exit(1);
        });

    if let Err(err) = db::run_migrations(&pool).await {
        tracing::error!("Failed to run database migrations: {}", err);
        process::exit(1);
    }

    let app = routes::create_routes(pool)
        .layer(axum_server::middleware::create_cors_layer())
        .layer(axum_server::middleware::create_trace_layer())
        .layer(axum::middleware::from_fn(axum_server::middleware::request_logging));

    let listener = TcpListener::bind(&config.server_address())
        .await
        .unwrap_or_else(|err| {
            tracing::error!("Failed to bind to {}: {}", config.server_address(), err);
            process::exit(1);
        });

    tracing::info!("Server running at http://{}", config.server_address());

    axum::serve(listener, app).await.unwrap_or_else(|err| {
        tracing::error!("Server error: {}", err);
        process::exit(1);
    });
}
