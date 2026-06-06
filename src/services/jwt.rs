use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,  // subject (User ID)
    pub exp: usize, // Expiration time (UTC timestamp)
    pub iat: usize, // Issued at (UTC timestamp)
}

pub fn generate_access_token(
    user_id: Uuid,
    secret: &str,
    duration_minutes: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::minutes(duration_minutes as i64);

    let claims = Claims {
        sub: user_id,
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
