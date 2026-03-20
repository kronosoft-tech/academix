//! Session Repository Port

use crate::domain::entities::Session;
use crate::domain::errors::DomainError;

/// Session repository port - defines operations for session persistence
pub trait SessionRepository: Send + Sync {
    /// Find session by ID
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError>;

    /// Find session by token
    fn find_by_token(&self, token: &str) -> Result<Option<Session>, DomainError>;

    /// Save a new session
    fn save(&self, session: &Session) -> Result<(), DomainError>;

    /// Delete session (logout)
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// Delete expired sessions (cleanup)
    fn delete_expired(&self) -> Result<u64, DomainError>;
}
