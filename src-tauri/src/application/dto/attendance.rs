//! Attendance DTOs

use serde::{Deserialize, Serialize};

/// Create attendance request
#[derive(Debug, Deserialize)]
pub struct CreateAttendanceRequest {
    pub student_id: String,
    pub group_id: String,
    pub date: String,
    pub status: String,
    pub notes: Option<String>,
}

/// Update attendance request
#[derive(Debug, Deserialize)]
pub struct UpdateAttendanceRequest {
    pub status: Option<String>,
    pub notes: Option<String>,
}

/// Attendance DTO
#[derive(Debug, Serialize)]
pub struct AttendanceDto {
    pub id: String,
    pub student_id: String,
    pub group_id: String,
    pub date: String,
    pub status: String,
    pub notes: Option<String>,
}

/// Group attendance statistics DTO
#[derive(Debug, Serialize)]
pub struct GroupAttendanceStats {
    pub group_id: String,
    pub total_records: i32,
    pub present_count: i32,
    pub present_percentage: f64,
    pub absent_count: i32,
    pub absent_percentage: f64,
    pub late_count: i32,
    pub late_percentage: f64,
    pub excused_count: i32,
    pub excused_percentage: f64,
    pub total_students: i32,
    pub total_sessions: i32,
}

/// Student absence count DTO
#[derive(Debug, Serialize)]
pub struct StudentAbsenceCountDto {
    pub student_id: String,
    pub absence_count: i32,
}

/// At-risk student DTO
#[derive(Debug, Serialize)]
pub struct AtRiskStudentDto {
    pub student_id: String,
    pub student_name: String,
    pub absence_count: i32,
}
