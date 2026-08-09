//! Course Repository Port

use async_trait::async_trait;
use crate::domain::entities::Course;
use crate::domain::errors::DomainError;

/// Course repository port - defines operations for course persistence
#[async_trait]
pub trait CourseRepository: Send + Sync {
    /// Find course by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError>;

    /// Find course by code
    async fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError>;

    /// Save a new course
    async fn save(&self, course: &Course) -> Result<(), DomainError>;

    /// Update an existing course
    async fn update(&self, course: &Course) -> Result<(), DomainError>;

    /// Delete course (soft delete - marks as archived)
    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all courses
    async fn find_all(&self) -> Result<Vec<Course>, DomainError>;

    /// List all archived courses
    async fn find_all_archived(&self) -> Result<Vec<Course>, DomainError>;

    /// Restore an archived course
    async fn restore(&self, id: &str) -> Result<(), DomainError>;

    /// Hard delete - permanently removes from database
    async fn hard_delete(&self, id: &str) -> Result<(), DomainError>;
}
