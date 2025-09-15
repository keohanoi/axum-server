use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    routes::AppState,
    error::{AppError, Result},
    models::{
        Category, CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest,
    },
};

#[derive(serde::Deserialize)]
pub struct CategoryQuery {
    pub user_id: Uuid,
}

pub async fn create_category(
    State(state): State<AppState>,
    Query(query): Query<CategoryQuery>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>)> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if category name already exists for this user
    let existing = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE name = $1 AND user_id = $2"
    )
    .bind(&payload.name)
    .bind(query.user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Category name already exists".to_string()));
    }

    let now = Utc::now();
    let category = sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories (name, description, color, user_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.color)
    .bind(query.user_id)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db_pool)
    .await?;

    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn get_categories(
    State(state): State<AppState>,
    Query(query): Query<CategoryQuery>,
) -> Result<Json<Vec<CategoryResponse>>> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE user_id = $1 ORDER BY name"
    )
    .bind(query.user_id)
    .fetch_all(&state.db_pool)
    .await?;

    let response: Vec<CategoryResponse> = categories.into_iter().map(CategoryResponse::from).collect();
    Ok(Json(response))
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<CategoryResponse>> {
    let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
        .bind(category_id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Category with id {} not found", category_id)))?;

    Ok(Json(category.into()))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let existing_category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
        .bind(category_id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Category with id {} not found", category_id)))?;

    let name = payload.name.as_ref().map(|n| n.clone()).unwrap_or(existing_category.name.clone());
    let description = payload.description.or(existing_category.description);
    let color = payload.color.or(existing_category.color);

    // Check if new name conflicts with existing categories for this user
    if let Some(ref new_name) = payload.name {
        if new_name != &existing_category.name {
            let existing = sqlx::query_as::<_, Category>(
                "SELECT * FROM categories WHERE name = $1 AND user_id = $2 AND id != $3"
            )
            .bind(new_name)
            .bind(existing_category.user_id)
            .bind(category_id)
            .fetch_optional(&state.db_pool)
            .await?;

            if existing.is_some() {
                return Err(AppError::Conflict("Category name already exists".to_string()));
            }
        }
    }

    let updated_category = sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories
        SET name = $1, description = $2, color = $3, updated_at = $4
        WHERE id = $5
        RETURNING *
        "#,
    )
    .bind(&name)
    .bind(&description)
    .bind(&color)
    .bind(Utc::now())
    .bind(category_id)
    .fetch_one(&state.db_pool)
    .await?;

    Ok(Json(updated_category.into()))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(category_id): Path<Uuid>,
) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(category_id)
        .execute(&state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Category with id {} not found", category_id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
