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

#[cfg(test)]
async fn error_response_parts(err: AppError) -> (StatusCode, String) {
    let resp = err.into_response();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("body should be available synchronously");
    let body_json: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("body should be valid JSON");
    let error_msg = body_json["error"].as_str().unwrap().to_string();
    (status, error_msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Error as SqlxError;

    #[tokio::test]
    async fn database_error_maps_to_500() {
        let err = AppError::Database(SqlxError::Configuration("test".into()));
        let (status, msg) = error_response_parts(err).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "internal server error");
    }

    #[tokio::test]
    async fn email_already_exists_maps_to_409() {
        let (status, msg) = error_response_parts(AppError::EmailAlreadyExists).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(msg, "email already exists");
    }

    #[tokio::test]
    async fn password_hashing_failed_maps_to_500() {
        let (status, msg) = error_response_parts(AppError::PasswordHashingFailed).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "password hashing failed");
    }

    #[tokio::test]
    async fn validation_error_maps_to_400() {
        let (status, msg) =
            error_response_parts(AppError::ValidationError("bad field".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(msg, "validation error");
    }

    #[tokio::test]
    async fn redis_error_maps_to_500() {
        let err = AppError::Redis(redis::RedisError::from((
            redis::ErrorKind::IoError,
            "connection refused",
        )));
        let (status, msg) = error_response_parts(err).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "session storage error");
    }

    #[tokio::test]
    async fn jwt_error_maps_to_500() {
        let err = AppError::Jwt(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
        let (status, msg) = error_response_parts(err).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "token generation error");
    }
}
