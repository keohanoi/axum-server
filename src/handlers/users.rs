use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::DbPool,
    error::{AppError, Result},
    models::{
        AuthResponse, CreateUserRequest, LoginRequest, UpdateUserRequest, User, UserResponse,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID
    pub username: String,
    pub exp: usize, // Expiration time
}

const JWT_SECRET: &[u8] = b"your-secret-key"; // In production, use environment variable

pub async fn register_user(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>)> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if username or email already exists
    let existing = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = $1 OR email = $2"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Username or email already exists".to_string()));
    }

    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    let now = Utc::now();
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, full_name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

pub async fn login_user(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("Account is disabled".to_string()));
    }

    let is_valid = verify(&payload.password, &user.password_hash)
        .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

    if !is_valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))?;

    Ok(Json(AuthResponse {
        user: user.into(),
        token,
    }))
}

pub async fn get_user_profile(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

    Ok(Json(user.into()))
}

pub async fn update_user_profile(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", user_id)))?;

    let email = payload.email.unwrap_or(existing_user.email);
    let full_name = payload.full_name.or(existing_user.full_name);
    let is_active = payload.is_active.unwrap_or(existing_user.is_active);

    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET email = $1, full_name = $2, is_active = $3, updated_at = $4
        WHERE id = $5
        RETURNING *
        "#,
    )
    .bind(&email)
    .bind(&full_name)
    .bind(is_active)
    .bind(Utc::now())
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated_user.into()))
}

pub async fn delete_user(
    State(pool): State<DbPool>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("User with id {} not found", user_id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
