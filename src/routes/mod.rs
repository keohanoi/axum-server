use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::{db::DbPool, handlers, kafka::EventProducer};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub kafka_producer: EventProducer,
}

pub fn create_routes(pool: DbPool, kafka_producer: EventProducer) -> Router {
    let state = AppState {
        db_pool: pool,
        kafka_producer,
    };
    Router::new()
        // Todo routes
        .route("/api/todos", post(handlers::create_todo))
        .route("/api/todos", get(handlers::get_todos))
        .route("/api/todos/{id}", get(handlers::get_todo))
        .route("/api/todos/{id}", patch(handlers::update_todo))
        .route("/api/todos/{id}", delete(handlers::delete_todo))
        
        // Batch operations - TODO: Update handlers for AppState
        // .route("/api/todos/batch", patch(handlers::batch::batch_update_todos))
        // .route("/api/todos/batch", delete(handlers::batch::batch_delete_todos))
        
        // User routes - TODO: Update handlers for AppState  
        // .route("/api/users/register", post(handlers::users::register_user))
        // .route("/api/users/login", post(handlers::users::login_user))
        // .route("/api/users/{id}", get(handlers::users::get_user_profile))
        // .route("/api/users/{id}", patch(handlers::users::update_user_profile))
        // .route("/api/users/{id}", delete(handlers::users::delete_user))
        
        // Category routes - TODO: Update handlers for AppState
        // .route("/api/categories", post(handlers::categories::create_category))
        // .route("/api/categories", get(handlers::categories::get_categories))
        // .route("/api/categories/{id}", get(handlers::categories::get_category))
        // .route("/api/categories/{id}", patch(handlers::categories::update_category))
        // .route("/api/categories/{id}", delete(handlers::categories::delete_category))
        
        // Tag routes - TODO: Update handlers for AppState
        // .route("/api/tags", post(handlers::tags::create_tag))
        // .route("/api/tags", get(handlers::tags::get_tags))
        // .route("/api/tags/{id}", get(handlers::tags::get_tag))
        // .route("/api/tags/{id}", delete(handlers::tags::delete_tag))
        // .route("/api/todos/{todo_id}/tags/{tag_id}", put(handlers::tags::assign_tag_to_todo))
        // .route("/api/todos/{todo_id}/tags/{tag_id}", delete(handlers::tags::remove_tag_from_todo))
        
        // Statistics routes - TODO: Update handlers for AppState
        // .route("/api/stats/todos", get(handlers::stats::get_todo_statistics))
        
        // Health check
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}