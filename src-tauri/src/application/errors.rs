//! Application Errors - Academix MVP
//!
//! Application-specific errors.

use thiserror::Error;

/// Application errors - represent application layer failures
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Domain error: {0}")]
    Domain(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl From<crate::domain::DomainError> for ApplicationError {
    fn from(err: crate::domain::DomainError) -> Self {
        match err {
            crate::domain::DomainError::NotFound(msg) => Self::NotFound(msg),
            crate::domain::DomainError::Validation(msg) => Self::Validation(msg),
            crate::domain::DomainError::Authentication(msg) => Self::Authentication(msg),
            crate::domain::DomainError::Authorization(msg) => Self::Authorization(msg),
            crate::domain::DomainError::InvalidOperation(msg) => Self::Domain(msg),
            crate::domain::DomainError::DuplicateEntry(msg) => Self::Conflict(msg),
            crate::domain::DomainError::Database(msg) => Self::Infrastructure(msg),
        }
    }
}

/// Result type alias for application operations
pub type ApplicationResult<T> = Result<T, ApplicationError>;
