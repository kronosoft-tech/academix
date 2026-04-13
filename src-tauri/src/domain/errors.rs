//! Domain Errors - Academix MVP
//!
//! Domain-specific errors using thiserror for ergonomic error handling.

use thiserror::Error;

/// Domain errors - represent business logic failures
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for DomainError {
    fn from(err: rusqlite::Error) -> Self {
        DomainError::Database(err.to_string())
    }
}

impl DomainError {
    /// Create a not found error
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::NotFound(format!("{} with id {} not found", entity, id))
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication(message.into())
    }

    /// Create an authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::Authorization(message.into())
    }

    /// Create an invalid operation error
    pub fn invalid_operation(message: impl Into<String>) -> Self {
        Self::InvalidOperation(message.into())
    }

    /// Create a duplicate entry error
    pub fn duplicate_entry(entity: &str, field: &str) -> Self {
        Self::DuplicateEntry(format!("{} with {} already exists", entity, field))
    }
}

/// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;
