use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    AppState,
    db::users::{CreateUserError, create_user},
    error::AppError,
    services::auth::{generate_verification_token, hash_password},
};

#[derive(Deserialize, Validate, Debug)]
pub struct SignUpRequest {
    #[validate(email(message = "invalid email format"))]
    pub email: String,

    #[validate(length(min = 5, message = "password must be at least 5 characters"))]
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct SignUpResponse {
    pub message: String,
    pub email: String,
}

pub async fn sign_up(
    State(state): State<AppState>,
    Json(payload): Json<SignUpRequest>,
) -> Result<(StatusCode, Json<SignUpResponse>), AppError> {
    if let Err(e) = payload.validate() {
        let error_message = e
            .field_errors()
            .values()
            .flat_map(|errors| {
                errors
                    .iter()
                    .map(|e| e.message.as_deref().unwrap_or("invalid"))
            })
            .collect::<Vec<_>>()
            .join(", ");

        return Err(AppError::ValidationError(error_message));
    }

    let password_hash =
        hash_password(&payload.password).map_err(|_| AppError::PasswordHashingFailed)?;

    let user = create_user(&state.db, &payload.email, &password_hash)
        .await
        .map_err(|e| match e {
            CreateUserError::EmailAlreadyExists => AppError::EmailAlreadyExists,
            CreateUserError::DatabaseError(e) => AppError::Database(e),
        })?;

    let _token = generate_verification_token();

    tracing::info!(
        "generated verification token for user {}: {}",
        user.email,
        _token
    );

    Ok((
        StatusCode::CREATED,
        Json(SignUpResponse {
            message: "user created successfully. Please check your email for verification."
                .to_string(),
            email: user.email,
        }),
    ))
}
