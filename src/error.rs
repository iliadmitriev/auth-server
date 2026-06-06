use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Password hashing failed")]
    PasswordHashingFailed,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Redis error")]
    Redis(#[from] redis::RedisError),

    #[error("Jwt Error")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
            AppError::EmailAlreadyExists => (StatusCode::CONFLICT, "email already exists"),
            AppError::PasswordHashingFailed => {
                (StatusCode::INTERNAL_SERVER_ERROR, "password hashing failed")
            }
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "validation error"),
            AppError::Redis(_) => (StatusCode::INTERNAL_SERVER_ERROR, "session storage error"),
            AppError::Jwt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "token generation error"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        tracing::error!("{}: {}", status, error_message);

        (status, body).into_response()
    }
}
