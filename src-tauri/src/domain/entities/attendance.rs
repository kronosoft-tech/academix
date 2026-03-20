//! Attendance Entity - Domain Model
//!
//! Pure domain entity with no persistence concerns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Attendance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttendanceStatus {
    Present,
    Absent,
    Late,
    Excused,
}

impl AttendanceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttendanceStatus::Present => "present",
            AttendanceStatus::Absent => "absent",
            AttendanceStatus::Late => "late",
            AttendanceStatus::Excused => "excused",
        }
    }

    pub fn from_str(s: &str) -> Option<AttendanceStatus> {
        match s.to_lowercase().as_str() {
            "present" => Some(AttendanceStatus::Present),
            "absent" => Some(AttendanceStatus::Absent),
            "late" => Some(AttendanceStatus::Late),
            "excused" => Some(AttendanceStatus::Excused),
            _ => None,
        }
    }
}

/// Attendance entity - represents student attendance in a group session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendance {
    pub id: String,
    pub student_id: String,
    pub group_id: String,
    pub date: DateTime<Utc>,
    pub status: AttendanceStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Attendance {
    /// Create a new attendance record
    pub fn new(
        id: String,
        student_id: String,
        group_id: String,
        date: DateTime<Utc>,
        status: AttendanceStatus,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            student_id,
            group_id,
            date,
            status,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark student as present
    pub fn mark_present(&mut self) {
        self.status = AttendanceStatus::Present;
        self.updated_at = Utc::now();
    }

    /// Mark student as absent
    pub fn mark_absent(&mut self) {
        self.status = AttendanceStatus::Absent;
        self.updated_at = Utc::now();
    }

    /// Mark student as late
    pub fn mark_late(&mut self) {
        self.status = AttendanceStatus::Late;
        self.updated_at = Utc::now();
    }

    /// Mark attendance as excused
    pub fn excuse(&mut self, notes: Option<String>) {
        self.status = AttendanceStatus::Excused;
        self.notes = notes;
        self.updated_at = Utc::now();
    }

    /// Check if student was present (present or late)
    pub fn was_present(&self) -> bool {
        matches!(
            self.status,
            AttendanceStatus::Present | AttendanceStatus::Late
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_attendance_creation() {
        let now = Utc::now();
        let attendance = Attendance::new(
            "attendance-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Present,
        );

        assert_eq!(attendance.id, "attendance-1");
        assert_eq!(attendance.student_id, "student-1");
        assert_eq!(attendance.group_id, "group-1");
        assert_eq!(attendance.status, AttendanceStatus::Present);
    }

    #[test]
    fn test_mark_present() {
        let now = Utc::now();
        let mut attendance = Attendance::new(
            "attendance-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Absent,
        );

        attendance.mark_present();

        assert_eq!(attendance.status, AttendanceStatus::Present);
    }

    #[test]
    fn test_mark_absent() {
        let now = Utc::now();
        let mut attendance = Attendance::new(
            "attendance-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Present,
        );

        attendance.mark_absent();

        assert_eq!(attendance.status, AttendanceStatus::Absent);
    }

    #[test]
    fn test_mark_late() {
        let now = Utc::now();
        let mut attendance = Attendance::new(
            "attendance-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Absent,
        );

        attendance.mark_late();

        assert_eq!(attendance.status, AttendanceStatus::Late);
    }

    #[test]
    fn test_excuse() {
        let now = Utc::now();
        let mut attendance = Attendance::new(
            "attendance-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Absent,
        );

        attendance.excuse(Some("Doctor's appointment".to_string()));

        assert_eq!(attendance.status, AttendanceStatus::Excused);
        assert_eq!(attendance.notes, Some("Doctor's appointment".to_string()));
    }

    #[test]
    fn test_was_present() {
        let now = Utc::now();

        let present = Attendance::new(
            "1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Present,
        );

        let late = Attendance::new(
            "2".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Late,
        );

        let absent = Attendance::new(
            "3".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Absent,
        );

        let excused = Attendance::new(
            "4".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            now,
            AttendanceStatus::Excused,
        );

        assert!(present.was_present());
        assert!(late.was_present());
        assert!(!absent.was_present());
        assert!(!excused.was_present());
    }

    #[test]
    fn test_attendance_status_from_str() {
        assert_eq!(
            AttendanceStatus::from_str("present"),
            Some(AttendanceStatus::Present)
        );
        assert_eq!(
            AttendanceStatus::from_str("ABSENT"),
            Some(AttendanceStatus::Absent)
        );
        assert_eq!(
            AttendanceStatus::from_str("late"),
            Some(AttendanceStatus::Late)
        );
        assert_eq!(
            AttendanceStatus::from_str("excused"),
            Some(AttendanceStatus::Excused)
        );
        assert_eq!(AttendanceStatus::from_str("unknown"), None);
    }
}
