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
        Tag, TagResponse, CreateTagRequest,
    },
};

#[derive(serde::Deserialize)]
pub struct TagQuery {
    pub user_id: Uuid,
}

pub async fn create_tag(
    State(pool): State<DbPool>,
    Query(query): Query<TagQuery>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<TagResponse>)> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if tag name already exists for this user
    let existing = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE name = $1 AND user_id = $2"
    )
    .bind(&payload.name)
    .bind(query.user_id)
    .fetch_optional(&pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Tag name already exists".to_string()));
    }

    let tag = sqlx::query_as::<_, Tag>(
        r#"
        INSERT INTO tags (name, user_id, created_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(query.user_id)
    .bind(Utc::now())
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(tag.into())))
}

pub async fn get_tags(
    State(pool): State<DbPool>,
    Query(query): Query<TagQuery>,
) -> Result<Json<Vec<TagResponse>>> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE user_id = $1 ORDER BY name"
    )
    .bind(query.user_id)
    .fetch_all(&pool)
    .await?;

    let response: Vec<TagResponse> = tags.into_iter().map(TagResponse::from).collect();
    Ok(Json(response))
}

pub async fn get_tag(
    State(pool): State<DbPool>,
    Path(tag_id): Path<Uuid>,
) -> Result<Json<TagResponse>> {
    let tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = $1")
        .bind(tag_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Tag with id {} not found", tag_id)))?;

    Ok(Json(tag.into()))
}

pub async fn delete_tag(
    State(pool): State<DbPool>,
    Path(tag_id): Path<Uuid>,
) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Tag with id {} not found", tag_id)));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_tag_to_todo(
    State(pool): State<DbPool>,
    Path((todo_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    // Check if todo and tag exist
    let todo_exists = sqlx::query("SELECT 1 FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_optional(&pool)
        .await?
        .is_some();

    if !todo_exists {
        return Err(AppError::NotFound(format!("Todo with id {} not found", todo_id)));
    }

    let tag_exists = sqlx::query("SELECT 1 FROM tags WHERE id = $1")
        .bind(tag_id)
        .fetch_optional(&pool)
        .await?
        .is_some();

    if !tag_exists {
        return Err(AppError::NotFound(format!("Tag with id {} not found", tag_id)));
    }

    // Insert the relationship (ignore if it already exists)
    sqlx::query(
        "INSERT INTO todo_tags (todo_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(todo_id)
    .bind(tag_id)
    .execute(&pool)
    .await?;

    Ok(StatusCode::OK)
}

pub async fn remove_tag_from_todo(
    State(pool): State<DbPool>,
    Path((todo_id, tag_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM todo_tags WHERE todo_id = $1 AND tag_id = $2")
        .bind(todo_id)
        .bind(tag_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Tag assignment not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
