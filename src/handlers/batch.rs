use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    db::DbPool,
    error::{AppError, Result},
    models::{BatchUpdateTodosRequest, TodoResponse},
};

pub async fn batch_update_todos(
    State(pool): State<DbPool>,
    Json(payload): Json<BatchUpdateTodosRequest>,
) -> Result<Json<Vec<TodoResponse>>> {
    if payload.todo_ids.is_empty() {
        return Err(AppError::Validation("No todo IDs provided".to_string()));
    }

    if payload.todo_ids.len() > 100 {
        return Err(AppError::Validation("Too many todos (max 100)".to_string()));
    }

    let mut tx = pool.begin().await?;
    let mut updated_todos = Vec::new();

    for todo_id in &payload.todo_ids {
        // Get existing todo
        let existing_todo = match sqlx::query_as::<_, crate::models::Todo>("SELECT * FROM todos WHERE id = $1")
            .bind(todo_id)
            .fetch_optional(&mut *tx)
            .await? {
                Some(todo) => todo,
                None => continue, // Skip if todo doesn't exist
            };

        // Apply updates
        let completed = payload.completed.unwrap_or(existing_todo.completed);
        let category_id = payload.category_id.or(existing_todo.category_id);
        let priority = payload.priority.or(existing_todo.priority);

        let updated_todo = sqlx::query_as::<_, crate::models::Todo>(
            "UPDATE todos SET completed = $1, category_id = $2, priority = $3, updated_at = $4 WHERE id = $5 RETURNING *"
        )
        .bind(completed)
        .bind(category_id)
        .bind(priority)
        .bind(Utc::now())
        .bind(todo_id)
        .fetch_one(&mut *tx)
        .await?;

        // Get full todo with relations
        let full_todo = get_todo_with_relations(&mut *tx, updated_todo.id).await?;
        updated_todos.push(full_todo);
    }

    tx.commit().await?;

    Ok(Json(updated_todos))
}

pub async fn batch_delete_todos(
    State(pool): State<DbPool>,
    Json(todo_ids): Json<Vec<Uuid>>,
) -> Result<StatusCode> {
    if todo_ids.is_empty() {
        return Err(AppError::Validation("No todo IDs provided".to_string()));
    }

    if todo_ids.len() > 100 {
        return Err(AppError::Validation("Too many todos (max 100)".to_string()));
    }

    let placeholders: Vec<String> = (1..=todo_ids.len())
        .map(|i| format!("${}", i))
        .collect();

    let query = format!(
        "DELETE FROM todos WHERE id IN ({})",
        placeholders.join(", ")
    );

    let mut q = sqlx::query(&query);
    for id in &todo_ids {
        q = q.bind(id);
    }

    let result = q.execute(&pool).await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("No todos found to delete".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Helper function to get todo with related data
async fn get_todo_with_relations(
    executor: &mut sqlx::PgConnection,
    todo_id: Uuid,
) -> Result<TodoResponse> {
    use crate::models::{Todo, Category, Tag, CategoryResponse, TagResponse};

    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_one(&mut *executor)
        .await?;

    let category = if let Some(category_id) = todo.category_id {
        sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
            .bind(category_id)
            .fetch_optional(&mut *executor)
            .await?
            .map(CategoryResponse::from)
    } else {
        None
    };

    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.* FROM tags t
        JOIN todo_tags tt ON t.id = tt.tag_id
        WHERE tt.todo_id = $1
        ORDER BY t.name
        "#
    )
    .bind(todo_id)
    .fetch_all(&mut *executor)
    .await?;

    let tag_responses: Vec<TagResponse> = tags.into_iter().map(TagResponse::from).collect();

    Ok(TodoResponse {
        id: todo.id,
        title: todo.title,
        description: todo.description,
        completed: todo.completed,
        user_id: todo.user_id,
        category,
        priority: todo.priority,
        due_date: todo.due_date,
        tags: tag_responses,
        created_at: todo.created_at,
        updated_at: todo.updated_at,
    })
}
