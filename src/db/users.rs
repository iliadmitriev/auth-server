#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateUserError {
    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<User, CreateUserError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id, email, password_hash, is_verified, created_at
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await;

    match user {
        Ok(u) => Ok(u),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(CreateUserError::EmailAlreadyExists)
        }
        Err(e) => Err(CreateUserError::DatabaseError(e)),
    }
}
