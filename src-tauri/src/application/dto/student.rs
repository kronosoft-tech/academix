//! Student DTOs

use serde::{Deserialize, Serialize};

/// Create student request
#[derive(Debug, Deserialize)]
pub struct CreateStudentRequest {
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub document_type: String,
    pub document_number: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub birth_date: Option<String>,
    pub guardian_name: Option<String>,
    pub guardian_document: Option<String>,
    pub guardian_phone: Option<String>,
    pub course_id: Option<String>,
    pub group_id: Option<String>,
}

/// Update student request
#[derive(Debug, Deserialize)]
pub struct UpdateStudentRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub birth_date: Option<String>,
    pub guardian_name: Option<String>,
    pub guardian_document: Option<String>,
    pub guardian_phone: Option<String>,
    pub course_id: Option<String>,
    pub group_id: Option<String>,
}

/// Student DTO
#[derive(Debug, Serialize)]
pub struct StudentDto {
    pub id: String,
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub document_type: String,
    pub document_number: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub birth_date: Option<String>,
    pub guardian_name: Option<String>,
    pub guardian_document: Option<String>,
    pub guardian_phone: Option<String>,
    pub course_id: Option<String>,
    pub group_id: Option<String>,
    pub course_name: Option<String>,
    pub group_name: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}
