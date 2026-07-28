//! User Repository Port

use async_trait::async_trait;
use crate::domain::entities::User;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Email;

/// User repository port - defines operations for user persistence
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find user by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError>;

    /// Find user by email
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;

    /// Save a new user
    async fn save(&self, user: &User) -> Result<(), DomainError>;

    /// Update an existing user
    async fn update(&self, user: &User) -> Result<(), DomainError>;

    /// Delete user (soft delete)
    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all users
    async fn find_all(&self) -> Result<Vec<User>, DomainError>;

    /// Check if email exists
    async fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError>;
}
