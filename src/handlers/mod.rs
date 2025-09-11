use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::DbPool,
    error::{AppError, Result},
    models::{
        CreateTodoRequest, Todo, TodoListResponse, TodoQuery, TodoResponse, UpdateTodoRequest,
    },
};

pub async fn create_todo(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<TodoResponse>)> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let now = Utc::now();
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (title, description, completed, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(false)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(todo.into())))
}

pub async fn get_todos(
    State(pool): State<DbPool>,
    Query(params): Query<TodoQuery>,
) -> Result<Json<TodoListResponse>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let mut query = String::from("SELECT * FROM todos");
    let mut count_query = String::from("SELECT COUNT(*) FROM todos");
    let mut conditions = Vec::new();
    let mut query_params = Vec::new();
    let mut param_index = 1;

    if let Some(completed) = params.completed {
        conditions.push(format!("completed = ${}", param_index));
        query_params.push(completed.to_string());
        param_index += 1;
    }

    if let Some(search) = &params.search {
        if !search.trim().is_empty() {
            conditions.push(format!("(title ILIKE ${} OR description ILIKE ${})", param_index, param_index));
            query_params.push(format!("%{}%", search));
        }
    }

    if !conditions.is_empty() {
        let where_clause = format!(" WHERE {}", conditions.join(" AND "));
        query.push_str(&where_clause);
        count_query.push_str(&where_clause);
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", per_page, offset));

    let total: i64 = if query_params.is_empty() {
        sqlx::query_scalar("SELECT COUNT(*) FROM todos")
            .fetch_one(&pool)
            .await?
    } else {
        let mut count_q = sqlx::query_scalar(&count_query);
        for param in &query_params {
            if param == "true" || param == "false" {
                count_q = count_q.bind(param.parse::<bool>().unwrap());
            } else {
                count_q = count_q.bind(param);
            }
        }
        count_q.fetch_one(&pool).await?
    };

    let todos: Vec<Todo> = if query_params.is_empty() {
        sqlx::query_as(&query).fetch_all(&pool).await?
    } else {
        let mut q = sqlx::query_as(&query);
        for param in &query_params {
            if param == "true" || param == "false" {
                q = q.bind(param.parse::<bool>().unwrap());
            } else {
                q = q.bind(param);
            }
        }
        q.fetch_all(&pool).await?
    };

    let response = TodoListResponse {
        todos: todos.into_iter().map(TodoResponse::from).collect(),
        total,
        page,
        per_page,
    };

    Ok(Json(response))
}

pub async fn get_todo(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TodoResponse>> {
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    Ok(Json(todo.into()))
}

pub async fn update_todo(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodoRequest>,
) -> Result<Json<TodoResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let existing_todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    let title = payload.title.unwrap_or(existing_todo.title);
    let description = payload.description.or(existing_todo.description);
    let completed = payload.completed.unwrap_or(existing_todo.completed);

    let updated_todo = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET title = $1, description = $2, completed = $3, updated_at = $4
        WHERE id = $5
        RETURNING *
        "#,
    )
    .bind(&title)
    .bind(&description)
    .bind(completed)
    .bind(Utc::now())
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated_todo.into()))
}

pub async fn delete_todo(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Todo with id {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}