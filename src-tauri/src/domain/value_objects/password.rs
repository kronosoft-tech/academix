//! Password Value Object
//!
//! Represents a password with validation rules.

use serde::{Deserialize, Serialize};

/// Password value object - validated password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Password {
    hash: String,
}

impl Password {
    /// Create a password from a pre-hashed value
    pub fn from_hash(hash: impl Into<String>) -> Self {
        Self { hash: hash.into() }
    }

    /// Get the hash
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Validate password strength (not hashed, for pre-hashing validation)
    pub fn validate_strength(password: &str) -> Result<(), String> {
        if password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }

        if password.len() > 128 {
            return Err("Password must be less than 128 characters".to_string());
        }

        Ok(())
    }
}

impl std::fmt::Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
