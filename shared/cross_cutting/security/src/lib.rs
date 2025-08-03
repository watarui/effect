//! セキュリティ関連のユーティリティ
//!
//! 認証、暗号化、トークン生成など

use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// セキュリティエラー
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Password hashing failed: {0}")]
    HashingError(String),

    #[error("Password verification failed")]
    VerificationError,

    #[error("JWT generation failed: {0}")]
    JwtGenerationError(String),

    #[error("JWT validation failed: {0}")]
    JwtValidationError(String),

    #[error("Invalid token")]
    InvalidToken,
}

/// JWT クレーム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub:  String, // Subject (user ID)
    pub exp:  u64,    // Expiration time
    pub iat:  u64,    // Issued at
    pub role: String, // User role
}

/// パスワードをハッシュ化
pub fn hash_password(password: &str) -> Result<String, SecurityError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| SecurityError::HashingError(e.to_string()))
}

/// パスワードを検証
pub fn verify_password(password: &str, hash: &str) -> Result<bool, SecurityError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| SecurityError::VerificationError)?;

    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map(|_| true)
        .or(Ok(false))
}

/// JWT トークンを生成
pub fn generate_jwt(
    user_id: &str,
    role: &str,
    secret: &str,
    expiration_hours: u64,
) -> Result<String, SecurityError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| SecurityError::JwtGenerationError(e.to_string()))?
        .as_secs();

    let claims = Claims {
        sub:  user_id.to_string(),
        exp:  now + (expiration_hours * 3600),
        iat:  now,
        role: role.to_string(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| SecurityError::JwtGenerationError(e.to_string()))
}

/// JWT トークンを検証
pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, SecurityError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|e| SecurityError::JwtValidationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let user_id = "user123";
        let role = "user";
        let secret = "test_secret";

        let token = generate_jwt(user_id, role, secret, 1).unwrap();
        let claims = validate_jwt(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, role);
    }
}
