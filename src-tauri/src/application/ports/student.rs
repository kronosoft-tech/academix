//! Student Repository Port

use crate::domain::entities::Student;
use crate::domain::errors::DomainError;

/// Student repository port - defines operations for student persistence
pub trait StudentRepository: Send + Sync {
    /// Find student by ID
    fn find_by_id(&self, id: &str) -> Result<Option<Student>, DomainError>;

    /// Find student by user ID
    fn find_by_user_id(&self, user_id: &str) -> Result<Option<Student>, DomainError>;

    /// Save a new student
    fn save(&self, student: &Student) -> Result<(), DomainError>;

    /// Update an existing student
    fn update(&self, student: &Student) -> Result<(), DomainError>;

    /// Delete student (soft delete)
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all students
    fn find_all(&self) -> Result<Vec<Student>, DomainError>;
}
