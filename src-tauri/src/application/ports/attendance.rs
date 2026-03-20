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
}
