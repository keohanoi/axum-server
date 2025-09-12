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
    pub user_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// Request/Response models
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTodoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    #[validate(range(min = 0, max = 4))]
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTodoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub category_id: Option<Uuid>,
    #[validate(range(min = 0, max = 4))]
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(max = 255))]
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 255))]
    pub full_name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(length(min = 7, max = 7))] // Hex color validation
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(length(min = 7, max = 7))] // Hex color validation
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchUpdateTodosRequest {
    pub todo_ids: Vec<Uuid>,
    pub completed: Option<bool>,
    pub category_id: Option<Uuid>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TodoResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub user_id: Option<Uuid>,
    pub category: Option<CategoryResponse>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<TagResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct TodoStatsResponse {
    pub total_todos: i64,
    pub completed_todos: i64,
    pub pending_todos: i64,
    pub overdue_todos: i64,
    pub todos_by_priority: Vec<PriorityCount>,
    pub todos_by_category: Vec<CategoryCount>,
}

#[derive(Debug, Serialize)]
pub struct PriorityCount {
    pub priority: i32,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct CategoryCount {
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub count: i64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

impl From<Category> for CategoryResponse {
    fn from(category: Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            description: category.description,
            color: category.color,
            created_at: category.created_at,
            updated_at: category.updated_at,
        }
    }
}

impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            created_at: tag.created_at,
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
    pub category_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub tag: Option<String>,
    pub overdue: Option<bool>,
}