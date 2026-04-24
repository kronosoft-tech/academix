//! Student Commands
//!
//! Tauri commands for student management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CreateStudentRequest, StudentDto, UpdateStudentRequest};
use crate::application::use_cases::StudentService;
use crate::infrastructure::repositories::{SqliteGroupRepository, SqliteStudentRepository};

pub type StudentServiceState = StudentService<SqliteStudentRepository, SqliteGroupRepository>;

/// Create student request payload
#[derive(Debug, Deserialize)]
pub struct CreateStudentCommand {
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

/// Update student request payload
#[derive(Debug, Deserialize)]
pub struct UpdateStudentCommand {
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

/// Student response payload
#[derive(Debug, Serialize)]
pub struct StudentCommandResponse {
    pub success: bool,
    pub data: Option<StudentDto>,
    pub error: Option<String>,
}

/// Student list response
#[derive(Debug, Serialize)]
pub struct StudentListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<StudentDto>>,
    pub error: Option<String>,
}

/// Create student command
#[tauri::command]
pub fn create_student(
    state: State<StudentServiceState>,
    request: CreateStudentCommand,
) -> StudentCommandResponse {
    match state.create(CreateStudentRequest {
        user_id: request.user_id,
        first_name: request.first_name,
        last_name: request.last_name,
        document_type: request.document_type,
        document_number: request.document_number,
        email: request.email,
        phone: request.phone,
        address: request.address,
        birth_date: request.birth_date,
        guardian_name: request.guardian_name,
        guardian_document: request.guardian_document,
        guardian_phone: request.guardian_phone,
        course_id: request.course_id,
        group_id: request.group_id,
    }) {
        Ok(student) => StudentCommandResponse {
            success: true,
            data: Some(student),
            error: None,
        },
        Err(e) => StudentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Get student by ID
#[tauri::command]
pub fn get_student(state: State<StudentServiceState>, id: String) -> StudentCommandResponse {
    match state.get_by_id(&id) {
        Ok(student) => StudentCommandResponse {
            success: true,
            data: Some(student),
            error: None,
        },
        Err(e) => StudentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List all students
#[tauri::command]
pub fn list_students(state: State<StudentServiceState>) -> StudentListCommandResponse {
    match state.list() {
        Ok(students) => StudentListCommandResponse {
            success: true,
            data: Some(students),
            error: None,
        },
        Err(e) => StudentListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Update student
#[tauri::command]
pub fn update_student(
    state: State<StudentServiceState>,
    id: String,
    request: UpdateStudentCommand,
) -> StudentCommandResponse {
    match state.update(
        &id,
        UpdateStudentRequest {
            first_name: request.first_name,
            last_name: request.last_name,
            email: request.email,
            phone: request.phone,
            address: request.address,
            birth_date: request.birth_date,
            guardian_name: request.guardian_name,
            guardian_document: request.guardian_document,
            guardian_phone: request.guardian_phone,
            course_id: request.course_id,
            group_id: request.group_id,
        },
    ) {
        Ok(student) => StudentCommandResponse {
            success: true,
            data: Some(student),
            error: None,
        },
        Err(e) => StudentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Delete student
#[tauri::command]
pub fn delete_student(state: State<StudentServiceState>, id: String) -> StudentCommandResponse {
    match state.delete(&id) {
        Ok(()) => StudentCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => StudentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}
