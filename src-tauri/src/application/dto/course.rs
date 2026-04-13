//! Course DTOs

use serde::{Deserialize, Serialize};

/// Create course request
#[derive(Debug, Deserialize)]
pub struct CreateCourseRequest {
    pub name: String,
    pub code: String,
    pub credits: i32,
    pub description: Option<String>,
    pub price: f64,
    pub duration: i32,
}

/// Update course request
#[derive(Debug, Deserialize)]
pub struct UpdateCourseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub credits: Option<i32>,
    pub price: Option<f64>,
    pub duration: Option<i32>,
}

/// Course DTO
#[derive(Debug, Serialize)]
pub struct CourseDto {
    pub id: String,
    pub name: String,
    pub code: String,
    pub credits: i32,
    pub price: f64,
    pub duration: i32,
    pub description: Option<String>,
    pub status: String,
}
