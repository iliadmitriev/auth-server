use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    AppState,
    db::users::{CreateUserError, GetUserError, create_user, get_user_by_email},
    error::AppError,
    services::{
        auth::{generate_verification_token, hash_password, verify_password},
        jwt::generate_access_token,
        session::{create_session, delete_session, get_session_user_id},
    },
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

#[derive(Deserialize, Debug, Validate)]
pub struct SignInRequest {
    #[validate(email(message = "invalid email format"))]
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct SignInResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(payload): Json<SignInRequest>,
) -> Result<(StatusCode, Json<SignInResponse>), AppError> {
    // 1.validate
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

    // 2. fetch
    let user = get_user_by_email(&state.db, &payload.email)
        .await
        .map_err(|e| match e {
            GetUserError::UserNotFound => {
                AppError::ValidationError("invalid email or password".to_string())
            }
            GetUserError::DatabaseError(e) => AppError::Database(e),
        })?;

    // 3. verify password
    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::ValidationError(
            "invalid email or password".to_string(),
        ));
    }

    // 4. generate token
    let access_token = generate_access_token(
        user.id,
        &state.jwt_secret,
        state.jwt_access_duration_minutes,
    )
    .map_err(AppError::Jwt)?;

    // 5. create session
    let session_id = create_session(
        state.redis.clone(),
        user.id,
        state.jwt_refresh_duration_days,
    )
    .await
    .map_err(AppError::Redis)?;

    Ok((
        StatusCode::OK,
        Json(SignInResponse {
            access_token,
            refresh_token: session_id,
            token_type: "Bearer".to_string(),
        }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub token_type: String,
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<RefreshTokenResponse>), AppError> {
    // check session
    let user_id = get_session_user_id(state.redis.clone(), &payload.refresh_token)
        .await
        .map_err(AppError::Redis)?;

    // get user_id
    let user_id = match user_id {
        Some(user_id) => user_id,
        None => {
            return Err(AppError::ValidationError(
                "invalid or expired session".to_string(),
            ));
        }
    };

    // generate new access token
    let access_token = generate_access_token(
        user_id,
        &state.jwt_secret,
        state.jwt_access_duration_minutes,
    )
    .map_err(AppError::Jwt)?;

    Ok((
        StatusCode::OK,
        Json(RefreshTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
        }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SignOutRequest {
    pub refresh_token: String,
}

pub async fn sign_out(
    State(state): State<AppState>,
    Json(payload): Json<SignOutRequest>,
) -> Result<StatusCode, AppError> {
    delete_session(state.redis.clone(), &payload.refresh_token)
        .await
        .map_err(AppError::Redis)?;

    Ok(StatusCode::OK)
}
