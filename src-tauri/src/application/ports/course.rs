//! Course Repository Port

use crate::domain::entities::Course;
use crate::domain::errors::DomainError;

/// Course repository port - defines operations for course persistence
pub trait CourseRepository: Send + Sync {
    /// Find course by ID
    fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError>;

    /// Find course by code
    fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError>;

    /// Save a new course
    fn save(&self, course: &Course) -> Result<(), DomainError>;

    /// Update an existing course
    fn update(&self, course: &Course) -> Result<(), DomainError>;

    /// Delete course (soft delete - marks as archived)
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all courses
    fn find_all(&self) -> Result<Vec<Course>, DomainError>;

    /// List all archived courses
    fn find_all_archived(&self) -> Result<Vec<Course>, DomainError>;

    /// Restore an archived course
    fn restore(&self, id: &str) -> Result<(), DomainError>;

    /// Hard delete - permanently removes from database
    fn hard_delete(&self, id: &str) -> Result<(), DomainError>;
}
