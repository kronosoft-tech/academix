//! Group DTOs

use serde::{Deserialize, Serialize};

/// Create group request
#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
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
    pub class_duration: Option<i32>,
    pub skipped_dates: Option<Vec<String>>,
}

/// Update group request
#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub professor_id: Option<String>,
    pub schedule: Option<String>,
    pub days: Option<Vec<String>>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub max_students: Option<i32>,
    pub status: Option<String>,
    pub class_duration: Option<i32>,
    pub skipped_dates: Option<Vec<String>>,
}

/// Group DTO
#[derive(Debug, Serialize)]
pub struct GroupDto {
    pub id: String,
    pub course_id: String,
    pub name: String,
    pub professor_id: String,
    pub schedule: Option<String>,
    pub days: Option<Vec<String>>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub max_students: i32,
    pub current_students: i32,
    pub status: String,
    pub class_duration: Option<i32>,
    pub skipped_dates: Vec<String>,
    pub calculated_end_date: Option<String>,
}
