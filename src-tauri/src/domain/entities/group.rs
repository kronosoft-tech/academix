//! Group Entity - Domain Model
//!
//! Pure domain entity with no persistence concerns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Group status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupStatus {
    Open,
    Closed,
    Completed,
}

impl GroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupStatus::Open => "open",
            GroupStatus::Closed => "closed",
            GroupStatus::Completed => "completed",
        }
    }

    pub fn from_str(s: &str) -> Option<GroupStatus> {
        match s.to_lowercase().as_str() {
            "open" => Some(GroupStatus::Open),
            "closed" => Some(GroupStatus::Closed),
            "completed" => Some(GroupStatus::Completed),
            _ => None,
        }
    }
}

/// Group entity - represents a student group taking a course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub course_id: String,
    pub name: String,
    pub professor_id: Option<String>,
    pub schedule: Option<String>,
    pub days: Option<Vec<String>>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub max_students: i32,
    pub current_students: i32,
    pub status: GroupStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub class_duration: Option<i32>,
    pub skipped_dates: Vec<String>,
}

impl Group {
    /// Create a new group
    pub fn new(
        id: String,
        course_id: String,
        name: String,
        professor_id: Option<String>,
        schedule: Option<String>,
        days: Option<Vec<String>>,
        start_time: Option<String>,
        end_time: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
        max_students: i32,
        class_duration: Option<i32>,
        skipped_dates: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            course_id,
            name,
            professor_id,
            schedule,
            days,
            start_time,
            end_time,
            start_date,
            end_date,
            max_students,
            current_students: 0,
            status: GroupStatus::Open,
            created_at: now,
            updated_at: now,
            class_duration,
            skipped_dates,
        }
    }

    /// Calculate end date based on course duration and class schedule
    /// Returns None if calculation is not possible (missing required data)
    pub fn calculate_end_date(&self, course_duration_hours: i32) -> Option<String> {
        // Edge cases: return None for missing data
        if course_duration_hours <= 0 {
            return None;
        }
        let class_duration = self.class_duration?;
        if class_duration <= 0 {
            return None;
        }
        let start_date_str = self.start_date.as_ref()?;
        let days = self.days.as_ref()?;
        if days.is_empty() {
            return None;
        }

        // Parse start date
        let start_date = chrono::NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d").ok()?;

        // Calculate total sessions needed: ceil((course_duration_hours * 60) / class_duration)
        let total_minutes = course_duration_hours as f64 * 60.0;
        let total_sessions_needed = (total_minutes / class_duration as f64).ceil() as i32;

        // Sessions per week = number of days in the schedule
        let sessions_per_week = days.len() as i32;

        // Weeks needed (without considering skipped dates)
        let weeks_needed = (total_sessions_needed as f64 / sessions_per_week as f64).ceil() as i32;

        // Count skipped dates that fall within the period
        // We need to check which skipped dates fall within the calculated period
        let mut effective_weeks = weeks_needed;
        let potential_end_date =
            start_date + chrono::Duration::weeks(weeks_needed as i64);

        for skipped_date_str in &self.skipped_dates {
            if let Ok(skipped_date) =
                chrono::NaiveDate::parse_from_str(skipped_date_str, "%Y-%m-%d")
            {
                // Check if the skipped date falls between start and the current potential end
                if skipped_date >= start_date && skipped_date < potential_end_date {
                    effective_weeks += 1;
                }
            }
        }

        let end_date = start_date + chrono::Duration::weeks(effective_weeks as i64);
        Some(end_date.format("%Y-%m-%d").to_string())
    }

    /// Add a student to the group
    pub fn add_student(&mut self) -> bool {
        if self.current_students < self.max_students && self.status == GroupStatus::Open {
            self.current_students += 1;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Remove a student from the group
    pub fn remove_student(&mut self) -> bool {
        if self.current_students > 0 {
            self.current_students -= 1;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Close the group (no more students can join)
    pub fn close(&mut self) {
        self.status = GroupStatus::Closed;
        self.updated_at = Utc::now();
    }

    /// Mark the group as completed
    pub fn complete(&mut self) {
        self.status = GroupStatus::Completed;
        self.updated_at = Utc::now();
    }

    /// Check if group has available spots
    pub fn has_capacity(&self) -> bool {
        self.current_students < self.max_students && self.status == GroupStatus::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_creation() {
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None, // professor_id
            None, // schedule
            None, // days
            None, // start_time
            None, // end_time
            None, // start_date
            None, // end_date
            30,   // max_students
            None, // class_duration
            vec![], // skipped_dates
        );

        assert_eq!(group.id, "group-1");
        assert_eq!(group.name, "Group A");
        assert_eq!(group.max_students, 30);
        assert_eq!(group.current_students, 0);
        assert_eq!(group.status, GroupStatus::Open);
    }

    #[test]
    fn test_add_student() {
        let mut group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            30,
            None,
            vec![],
        );

        assert!(group.add_student());
        assert_eq!(group.current_students, 1);

        // Add 29 more
        for _ in 0..29 {
            group.add_student();
        }
        assert_eq!(group.current_students, 30);

        // Should not exceed max
        assert!(!group.add_student());
    }

    #[test]
    fn test_remove_student() {
        let mut group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            30,
            None,
            vec![],
        );

        group.add_student();
        group.add_student();
        assert_eq!(group.current_students, 2);

        assert!(group.remove_student());
        assert_eq!(group.current_students, 1);

        // Cannot go below 0
        assert!(group.remove_student());
        assert_eq!(group.current_students, 0);
        assert!(!group.remove_student());
    }

    #[test]
    fn test_close_group() {
        let mut group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            30,
            None,
            vec![],
        );

        group.close();

        assert_eq!(group.status, GroupStatus::Closed);
        assert!(!group.has_capacity());
        assert!(!group.add_student());
    }

    #[test]
    fn test_has_capacity() {
        let mut group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            30,
            None,
            vec![],
        );

        assert!(group.has_capacity());

        // Fill the group
        for _ in 0..30 {
            group.add_student();
        }

        assert!(!group.has_capacity());
    }

    #[test]
    fn test_calculate_end_date_basic() {
        // 10 hour course, 2 hour classes, 2 days/week
        // Total sessions: 10*60/120 = 5 sessions
        // Sessions per week: 2
        // Weeks needed: ceil(5/2) = 3
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            Some(vec!["Lun".to_string(), "Mié".to_string()]),
            None,
            None,
            Some("2024-01-01".to_string()),
            None,
            30,
            Some(120), // 2 hours in minutes
            vec![],
        );

        let end_date = group.calculate_end_date(10);
        assert!(end_date.is_some());
        let end = end_date.unwrap();
        // 3 weeks from Jan 1, 2024 = Jan 22, 2024
        assert_eq!(end, "2024-01-22");
    }

    #[test]
    fn test_calculate_end_date_with_skipped_dates() {
        // Same as above but with 1 skipped date in the period
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            Some(vec!["Lun".to_string(), "Mié".to_string()]),
            None,
            None,
            Some("2024-01-01".to_string()),
            None,
            30,
            Some(120),
            vec!["2024-01-08".to_string()], // Skipped date in period
        );

        let end_date = group.calculate_end_date(10);
        assert!(end_date.is_some());
        let end = end_date.unwrap();
        // 3 weeks + 1 skipped = 4 weeks from Jan 1, 2024 = Jan 29, 2024
        assert_eq!(end, "2024-01-29");
    }

    #[test]
    fn test_calculate_end_date_no_start_date() {
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            Some(vec!["Lun".to_string()]),
            None,
            None,
            None, // No start date
            None,
            30,
            Some(60),
            vec![],
        );

        assert!(group.calculate_end_date(10).is_none());
    }

    #[test]
    fn test_calculate_end_date_no_days() {
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            None, // No days
            None,
            None,
            Some("2024-01-01".to_string()),
            None,
            30,
            Some(60),
            vec![],
        );

        assert!(group.calculate_end_date(10).is_none());
    }

    #[test]
    fn test_calculate_end_date_zero_course_duration() {
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            Some(vec!["Lun".to_string()]),
            None,
            None,
            Some("2024-01-01".to_string()),
            None,
            30,
            Some(60),
            vec![],
        );

        assert!(group.calculate_end_date(0).is_none());
    }

    #[test]
    fn test_calculate_end_date_no_class_duration() {
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            Some(vec!["Lun".to_string()]),
            None,
            None,
            Some("2024-01-01".to_string()),
            None,
            30,
            None, // No class duration
            vec![],
        );

        assert!(group.calculate_end_date(10).is_none());
    }

    #[test]
    fn test_calculate_end_date_multiple_skipped_dates() {
        // 5 hour course, 1 hour classes, 1 day/week
        // Total sessions: 5*60/60 = 5 sessions
        // Sessions per week: 1
        // Weeks needed: ceil(5/1) = 5
        // 2 skipped dates in period
        let group = Group::new(
            "group-1".to_string(),
            "course-1".to_string(),
            "Group A".to_string(),
            None,
            None,
            Some(vec!["Mar".to_string()]),
            None,
            None,
            Some("2024-01-02".to_string()),
            None,
            30,
            Some(60),
            vec![
                "2024-01-09".to_string(), // In period (week 1)
                "2024-01-16".to_string(), // In period (week 2)
            ],
        );

        let end_date = group.calculate_end_date(5);
        assert!(end_date.is_some());
        let end = end_date.unwrap();
        // 5 weeks + 2 skipped = 7 weeks from Jan 2 = Feb 20, 2024
        assert_eq!(end, "2024-02-20");
    }
}
