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
        Category, Tag, CategoryResponse, TagResponse,
    },
};

pub mod users;
pub mod categories;
pub mod tags;
pub mod stats;
pub mod batch;

// Helper function to get todo with related data
async fn get_todo_with_relations(
    pool: &DbPool,
    todo_id: Uuid,
) -> Result<TodoResponse> {
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_one(pool)
        .await?;

    let category = if let Some(category_id) = todo.category_id {
        sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
            .bind(category_id)
            .fetch_optional(pool)
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
    .fetch_all(pool)
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

pub async fn create_todo(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<TodoResponse>)> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let now = Utc::now();
    let todo = sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (title, description, completed, category_id, priority, due_date, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(false)
    .bind(&payload.category_id)
    .bind(&payload.priority)
    .bind(&payload.due_date)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await?;

    // Handle tags if provided
    if let Some(tag_names) = &payload.tags {
        for tag_name in tag_names {
            // For now, we'll assume user_id is required - this would be extracted from auth in real implementation
            let tag = sqlx::query_as::<_, Tag>(
                "INSERT INTO tags (name, user_id, created_at) VALUES ($1, $2, $3) 
                 ON CONFLICT (name, user_id) DO UPDATE SET name = EXCLUDED.name
                 RETURNING *"
            )
            .bind(tag_name)
            .bind(todo.user_id.unwrap_or_default()) // This should come from auth
            .bind(now)
            .fetch_one(&pool)
            .await?;

            // Link tag to todo
            sqlx::query("INSERT INTO todo_tags (todo_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(todo.id)
                .bind(tag.id)
                .execute(&pool)
                .await?;
        }
    }

    let todo_response = get_todo_with_relations(&pool, todo.id).await?;
    Ok((StatusCode::CREATED, Json(todo_response)))
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

    if let Some(category_id) = params.category_id {
        conditions.push(format!("category_id = ${}", param_index));
        query_params.push(category_id.to_string());
        param_index += 1;
    }

    if let Some(priority) = params.priority {
        conditions.push(format!("priority = ${}", param_index));
        query_params.push(priority.to_string());
        param_index += 1;
    }

    if let Some(search) = &params.search {
        if !search.trim().is_empty() {
            conditions.push(format!("(title ILIKE ${} OR description ILIKE ${})", param_index, param_index));
            query_params.push(format!("%{}%", search));
            param_index += 1;
        }
    }

    if let Some(tag) = &params.tag {
        conditions.push(format!(
            "id IN (SELECT tt.todo_id FROM todo_tags tt JOIN tags t ON tt.tag_id = t.id WHERE t.name ILIKE ${})",
            param_index
        ));
        query_params.push(format!("%{}%", tag));
        param_index += 1;
    }

    if params.overdue == Some(true) {
        conditions.push(format!("due_date < ${} AND completed = false", param_index));
        query_params.push(Utc::now().to_rfc3339());
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
            } else if let Ok(uuid) = param.parse::<Uuid>() {
                count_q = count_q.bind(uuid);
            } else if let Ok(num) = param.parse::<i32>() {
                count_q = count_q.bind(num);
            } else if let Ok(datetime) = param.parse::<chrono::DateTime<Utc>>() {
                count_q = count_q.bind(datetime);
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
            } else if let Ok(uuid) = param.parse::<Uuid>() {
                q = q.bind(uuid);
            } else if let Ok(num) = param.parse::<i32>() {
                q = q.bind(num);
            } else if let Ok(datetime) = param.parse::<chrono::DateTime<Utc>>() {
                q = q.bind(datetime);
            } else {
                q = q.bind(param);
            }
        }
        q.fetch_all(&pool).await?
    };

    // Convert todos with relations
    let mut todo_responses = Vec::new();
    for todo in todos {
        let todo_response = get_todo_with_relations(&pool, todo.id).await?;
        todo_responses.push(todo_response);
    }

    let response = TodoListResponse {
        todos: todo_responses,
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
    let todo_response = get_todo_with_relations(&pool, id).await
        .map_err(|_| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    Ok(Json(todo_response))
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
    let category_id = payload.category_id.or(existing_todo.category_id);
    let priority = payload.priority.or(existing_todo.priority);
    let due_date = payload.due_date.or(existing_todo.due_date);

    let updated_todo = sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET title = $1, description = $2, completed = $3, category_id = $4, 
            priority = $5, due_date = $6, updated_at = $7
        WHERE id = $8
        RETURNING *
        "#,
    )
    .bind(&title)
    .bind(&description)
    .bind(completed)
    .bind(&category_id)
    .bind(&priority)
    .bind(&due_date)
    .bind(Utc::now())
    .bind(id)
    .fetch_one(&pool)
    .await?;

    // Handle tags update if provided
    if let Some(tag_names) = &payload.tags {
        // Remove existing tags
        sqlx::query("DELETE FROM todo_tags WHERE todo_id = $1")
            .bind(id)
            .execute(&pool)
            .await?;

        // Add new tags
        for tag_name in tag_names {
            let tag = sqlx::query_as::<_, Tag>(
                "INSERT INTO tags (name, user_id, created_at) VALUES ($1, $2, $3) 
                 ON CONFLICT (name, user_id) DO UPDATE SET name = EXCLUDED.name
                 RETURNING *"
            )
            .bind(tag_name)
            .bind(updated_todo.user_id.unwrap_or_default())
            .bind(Utc::now())
            .fetch_one(&pool)
            .await?;

            sqlx::query("INSERT INTO todo_tags (todo_id, tag_id) VALUES ($1, $2)")
                .bind(id)
                .bind(tag.id)
                .execute(&pool)
                .await?;
        }
    }

    let todo_response = get_todo_with_relations(&pool, id).await?;
    Ok(Json(todo_response))
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