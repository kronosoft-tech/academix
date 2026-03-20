//! User Repository Port

use crate::domain::entities::User;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Email;

/// User repository port - defines operations for user persistence
pub trait UserRepository: Send + Sync {
    /// Find user by ID
    fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError>;

    /// Find user by email
    fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;

    /// Save a new user
    fn save(&self, user: &User) -> Result<(), DomainError>;

    /// Update an existing user
    fn update(&self, user: &User) -> Result<(), DomainError>;

    /// Delete user (soft delete)
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all users
    fn find_all(&self) -> Result<Vec<User>, DomainError>;

    /// Check if email exists
    fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError>;
}
