//! Student Repository Port

use async_trait::async_trait;
use crate::domain::entities::Student;
use crate::domain::errors::DomainError;

/// Student repository port - defines operations for student persistence
#[async_trait]
pub trait StudentRepository: Send + Sync {
    /// Find student by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Student>, DomainError>;

    /// Find student by user ID
    async fn find_by_user_id(&self, user_id: &str) -> Result<Option<Student>, DomainError>;

    /// Save a new student
    async fn save(&self, student: &Student) -> Result<(), DomainError>;

    /// Update an existing student
    async fn update(&self, student: &Student) -> Result<(), DomainError>;

    /// Delete student (soft delete)
    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all students
    async fn find_all(&self) -> Result<Vec<Student>, DomainError>;
}
