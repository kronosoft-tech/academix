//! Group Repository Port

use crate::domain::entities::Group;
use crate::domain::errors::DomainError;

/// Group repository port - defines operations for group persistence
pub trait GroupRepository: Send + Sync {
    /// Find group by ID
    fn find_by_id(&self, id: &str) -> Result<Option<Group>, DomainError>;

    /// Find groups by course ID
    fn find_by_course_id(&self, course_id: &str) -> Result<Vec<Group>, DomainError>;

    /// Find groups by professor ID
    fn find_by_professor_id(&self, professor_id: &str) -> Result<Vec<Group>, DomainError>;

    /// Save a new group
    fn save(&self, group: &Group) -> Result<(), DomainError>;

    /// Update an existing group
    fn update(&self, group: &Group) -> Result<(), DomainError>;

    /// Delete group
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all groups
    fn find_all(&self) -> Result<Vec<Group>, DomainError>;

    /// Check if a group has capacity for more students
    fn has_capacity(&self, group_id: &str) -> Result<bool, DomainError>;

    /// Increment the current students count for a group
    fn increment_students(&self, group_id: &str) -> Result<(), DomainError>;

    /// Decrement the current students count for a group
    fn decrement_students(&self, group_id: &str) -> Result<(), DomainError>;
}
