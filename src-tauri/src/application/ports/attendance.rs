//! Attendance Repository Port

use crate::domain::entities::Attendance;
use crate::domain::errors::DomainError;
use chrono::{DateTime, Utc};

/// Attendance repository port - defines operations for attendance persistence
pub trait AttendanceRepository: Send + Sync {
    /// Find attendance by ID
    fn find_by_id(&self, id: &str) -> Result<Option<Attendance>, DomainError>;

    /// Save a new attendance record
    fn save(&self, attendance: &Attendance) -> Result<(), DomainError>;

    /// Update an existing attendance record
    fn update(&self, attendance: &Attendance) -> Result<(), DomainError>;

    /// Delete attendance record
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all attendance records
    fn find_all(&self) -> Result<Vec<Attendance>, DomainError>;

    /// Find attendance by student ID
    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Attendance>, DomainError>;

    /// Find attendance by group ID and date
    fn find_by_group_and_date(
        &self,
        group_id: &str,
        date: DateTime<Utc>,
    ) -> Result<Vec<Attendance>, DomainError>;

    /// Count absences for a student in a specific group
    fn count_absences_by_student_and_group(
        &self,
        student_id: &str,
        group_id: &str,
    ) -> Result<i32, DomainError>;

    /// Count absences for all students in a group (returns student_id, count pairs)
    fn count_absences_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<(String, i32)>, DomainError>;
}
