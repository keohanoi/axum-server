use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{
    db::DbPool,
    handlers::users::Claims,
};

const JWT_SECRET: &[u8] = b"your-secret-key"; // In production, use environment variable

pub async fn auth_middleware(
    State(_pool): State<DbPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            match decode::<Claims>(
                token,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::default(),
            ) {
                Ok(token_data) => {
                    // Add user info to request extensions
                    request.extensions_mut().insert(token_data.claims);
                    return Ok(next.run(request).await);
                }
                Err(_) => return Err(StatusCode::UNAUTHORIZED),
            }
        }
    }

    // For now, allow requests without auth for backwards compatibility
    // In production, you'd return Err(StatusCode::UNAUTHORIZED) here
    Ok(next.run(request).await)
}
