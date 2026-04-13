//! Email Value Object
//!
//! Validated email address.

use serde::{Deserialize, Serialize};

/// Email value object - validated email address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Create a new email, validating the format
    pub fn new(email: impl Into<String>) -> Result<Self, String> {
        let email = email.into();

        // Basic email validation
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        if !email.contains('@') {
            return Err("Email must contain @".to_string());
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err("Email must have exactly one @".to_string());
        }

        if parts[0].is_empty() || parts[1].is_empty() {
            return Err("Email local and domain cannot be empty".to_string());
        }

        Ok(Self(email))
    }

    /// Get the email as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the domain part of the email
    pub fn domain(&self) -> Option<&str> {
        self.0.split('@').nth(1)
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
