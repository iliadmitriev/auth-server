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

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{DecodingKey, Validation, decode};

    #[test]
    fn generate_access_token_returns_decodable_jwt() {
        let user_id = Uuid::new_v4();
        let token = generate_access_token(user_id, "secret", 60).unwrap();
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("secret".as_bytes()),
            &Validation::default(),
        )
        .unwrap();
        assert_eq!(decoded.claims.sub, user_id);
    }

    #[test]
    fn claims_subject_matches_user_id() {
        let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let token = generate_access_token(user_id, "secret", 60).unwrap();
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("secret".as_bytes()),
            &Validation::default(),
        )
        .unwrap();
        assert_eq!(decoded.claims.sub, user_id);
    }

    #[test]
    fn claims_exp_is_after_iat() {
        let user_id = Uuid::new_v4();
        let token = generate_access_token(user_id, "secret", 60).unwrap();
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("secret".as_bytes()),
            &Validation::default(),
        )
        .unwrap();
        assert!(decoded.claims.exp > decoded.claims.iat);
    }

    #[test]
    fn claims_iat_is_close_to_now() {
        let user_id = Uuid::new_v4();
        let before = Utc::now().timestamp() as usize;
        let token = generate_access_token(user_id, "secret", 60).unwrap();
        let after = Utc::now().timestamp() as usize;
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("secret".as_bytes()),
            &Validation::default(),
        )
        .unwrap();
        assert!(decoded.claims.iat >= before && decoded.claims.iat <= after);
    }

    #[test]
    fn tokens_with_different_secrets_differ() {
        let user_id = Uuid::new_v4();
        let t1 = generate_access_token(user_id, "secret_a", 60).unwrap();
        let t2 = generate_access_token(user_id, "secret_b", 60).unwrap();
        assert_ne!(t1, t2);
    }

    #[test]
    fn tokens_with_different_user_ids_differ() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let t1 = generate_access_token(id1, "secret", 60).unwrap();
        let t2 = generate_access_token(id2, "secret", 60).unwrap();
        assert_ne!(t1, t2);
    }

    #[test]
    fn decode_with_wrong_secret_fails() {
        let user_id = Uuid::new_v4();
        let token = generate_access_token(user_id, "secret", 60).unwrap();
        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret("wrong".as_bytes()),
            &Validation::default(),
        );
        assert!(result.is_err());
    }
}
