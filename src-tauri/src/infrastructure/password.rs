//! Password Hashing - bcrypt
//!
//! Provides password hashing and verification using bcrypt.

use bcrypt::{hash, verify, BcryptResult};
use thiserror::Error;

/// Password hashing errors
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Failed to hash password: {0}")]
    HashError(String),

    #[error("Failed to verify password: {0}")]
    VerifyError(String),
}

/// Hash a password using bcrypt with default cost
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    hash_password_with_cost(password, 10)
}

/// Hash a password with a specific cost
pub fn hash_password_with_cost(password: &str, cost: u32) -> Result<String, PasswordError> {
    let result: BcryptResult<String> = hash(password, cost);

    match result {
        Ok(hashed) => Ok(hashed),
        Err(e) => Err(PasswordError::HashError(e.to_string())),
    }
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    match verify(password, hash) {
        Ok(valid) => valid,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123";
        let hashed = hash_password(password).unwrap();

        assert!(verify_password(password, &hashed));
        assert!(!verify_password("wrong_password", &hashed));
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "test_password_123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Different salts produce different hashes
        assert_ne!(hash1, hash2);

        // Both should verify
        assert!(verify_password(password, &hash1));
        assert!(verify_password(password, &hash2));
    }
}
