#![allow(dead_code)]

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use uuid::Uuid;

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(parsed_hash) => parsed_hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn generate_verification_token() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_returns_valid_argon2_hash() {
        let hash = hash_password("securepassword123").unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn verify_password_returns_true_for_correct_password() {
        let hash = hash_password("mysecret").unwrap();
        assert!(verify_password("mysecret", &hash));
    }

    #[test]
    fn verify_password_returns_false_for_wrong_password() {
        let hash = hash_password("mysecret").unwrap();
        assert!(!verify_password("wrongpassword", &hash));
    }

    #[test]
    fn verify_password_returns_false_for_malformed_hash() {
        assert!(!verify_password("any", "not-a-valid-hash"));
    }

    #[test]
    fn verify_password_returns_false_for_empty_password() {
        let hash = hash_password("mysecret").unwrap();
        assert!(!verify_password("", &hash));
    }

    #[test]
    fn generate_verification_token_returns_valid_uuid_v4() {
        let token = generate_verification_token();
        let parsed = Uuid::parse_str(&token);
        assert!(parsed.is_ok());
    }

    #[test]
    fn generate_verification_tokens_are_unique() {
        let t1 = generate_verification_token();
        let t2 = generate_verification_token();
        assert_ne!(t1, t2);
    }
}
