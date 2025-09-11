use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTodoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTodoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TodoResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Todo> for TodoResponse {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            completed: todo.completed,
            created_at: todo.created_at,
            updated_at: todo.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TodoListResponse {
    pub todos: Vec<TodoResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Deserialize)]
pub struct TodoQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub completed: Option<bool>,
    pub search: Option<String>,
}