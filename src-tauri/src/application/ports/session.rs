//! Session Repository Port

use async_trait::async_trait;
use crate::domain::entities::Session;
use crate::domain::errors::DomainError;

/// Session repository port - defines operations for session persistence
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Find session by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError>;

    /// Find session by token
    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, DomainError>;

    /// Save a new session
    async fn save(&self, session: &Session) -> Result<(), DomainError>;

    /// Delete session (logout)
    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// Delete expired sessions (cleanup)
    async fn delete_expired(&self) -> Result<u64, DomainError>;
}
