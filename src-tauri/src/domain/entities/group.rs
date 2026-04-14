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
        }
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
        );

        assert!(group.has_capacity());

        // Fill the group
        for _ in 0..30 {
            group.add_student();
        }

        assert!(!group.has_capacity());
    }
}
