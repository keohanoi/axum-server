use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::{db::DbPool, handlers};

pub fn create_routes(pool: DbPool) -> Router {
    Router::new()
        .route("/api/todos", post(handlers::create_todo))
        .route("/api/todos", get(handlers::get_todos))
        .route("/api/todos/{id}", get(handlers::get_todo))
        .route("/api/todos/{id}", patch(handlers::update_todo))
        .route("/api/todos/{id}", delete(handlers::delete_todo))
        .route("/health", get(health_check))
        .with_state(pool)
}

async fn health_check() -> &'static str {
    "OK"
}