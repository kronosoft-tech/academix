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
    pub max_students: i32,
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
    pub max_students: Option<i32>,
    pub status: Option<String>,
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
    pub max_students: i32,
    pub current_students: i32,
    pub status: String,
}
