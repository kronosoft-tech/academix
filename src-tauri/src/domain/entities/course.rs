//! Course Entity - Domain Model
//!
//! Pure domain entity with no persistence concerns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Course status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CourseStatus {
    Draft,
    Active,
    Archived,
}

impl CourseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseStatus::Draft => "draft",
            CourseStatus::Active => "active",
            CourseStatus::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Option<CourseStatus> {
        match s.to_lowercase().as_str() {
            "draft" => Some(CourseStatus::Draft),
            "active" => Some(CourseStatus::Active),
            "archived" => Some(CourseStatus::Archived),
            _ => None,
        }
    }
}

/// Course entity - represents an academic course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub credits: i32,
    pub price: f64,
    pub duration: i32, // in hours
    pub status: CourseStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Course {
    /// Create a new course
    pub fn new(id: String, name: String, code: String, credits: i32) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: None,
            code,
            credits,
            price: 200000.0,
            duration: 0,
            status: CourseStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }

    /// Activate the course
    pub fn activate(&mut self) {
        self.status = CourseStatus::Active;
        self.updated_at = Utc::now();
    }

    /// Archive the course
    pub fn archive(&mut self) {
        self.status = CourseStatus::Archived;
        self.updated_at = Utc::now();
    }

    /// Update course details
    pub fn update(
        &mut self,
        name: String,
        description: Option<String>,
        credits: i32,
        price: f64,
        duration: i32,
    ) {
        self.name = name;
        self.description = description;
        self.credits = credits;
        self.price = price;
        self.duration = duration;
        self.updated_at = Utc::now();
    }

    /// Check if course is active
    pub fn is_active(&self) -> bool {
        self.status == CourseStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_creation() {
        let course = Course::new(
            "course-1".to_string(),
            "Introduction to Programming".to_string(),
            "CS101".to_string(),
            4,
        );

        assert_eq!(course.id, "course-1");
        assert_eq!(course.name, "Introduction to Programming");
        assert_eq!(course.code, "CS101");
        assert_eq!(course.credits, 4);
        assert_eq!(course.status, CourseStatus::Draft);
    }

    #[test]
    fn test_activate_course() {
        let mut course = Course::new(
            "course-1".to_string(),
            "Course".to_string(),
            "CS101".to_string(),
            4,
        );

        assert_eq!(course.status, CourseStatus::Draft);

        course.activate();

        assert_eq!(course.status, CourseStatus::Active);
        assert!(course.is_active());
    }

    #[test]
    fn test_archive_course() {
        let mut course = Course::new(
            "course-1".to_string(),
            "Course".to_string(),
            "CS101".to_string(),
            4,
        );

        course.archive();

        assert_eq!(course.status, CourseStatus::Archived);
        assert!(!course.is_active());
    }

    #[test]
    fn test_update_course() {
        let mut course = Course::new(
            "course-1".to_string(),
            "Old Name".to_string(),
            "CS101".to_string(),
            4,
        );

        course.update(
            "New Name".to_string(),
            Some("New description".to_string()),
            6,
            150.0, // price
            30,    // duration
        );

        assert_eq!(course.name, "New Name");
        assert_eq!(course.description, Some("New description".to_string()));
        assert_eq!(course.credits, 6);
    }

    #[test]
    fn test_course_status_as_str() {
        assert_eq!(CourseStatus::Draft.as_str(), "draft");
        assert_eq!(CourseStatus::Active.as_str(), "active");
        assert_eq!(CourseStatus::Archived.as_str(), "archived");
    }

    #[test]
    fn test_course_status_from_str() {
        assert_eq!(CourseStatus::from_str("draft"), Some(CourseStatus::Draft));
        assert_eq!(CourseStatus::from_str("ACTIVE"), Some(CourseStatus::Active));
        assert_eq!(CourseStatus::from_str("unknown"), None);
    }
}
