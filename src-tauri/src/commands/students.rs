use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CreateStudentRequest, StudentDto, UpdateStudentRequest};
use crate::application::use_cases::StudentService;
use crate::infrastructure::repositories::{MemoryBackedGroupRepository, MemoryBackedStudentRepository};

pub type StudentServiceState = StudentService<MemoryBackedStudentRepository, MemoryBackedGroupRepository>;

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

#[derive(Debug, Serialize)]
pub struct StudentCommandResponse {
    pub success: bool,
    pub data: Option<StudentDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StudentListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<StudentDto>>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn create_student(
    state: State<'_, StudentServiceState>,
    request: CreateStudentCommand,
) -> Result<StudentCommandResponse, String> {
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
    }).await {
        Ok(student) => Ok(StudentCommandResponse {
            success: true,
            data: Some(student),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_student(state: State<'_, StudentServiceState>, id: String) -> Result<StudentCommandResponse, String> {
    match state.get_by_id(&id).await {
        Ok(student) => Ok(StudentCommandResponse {
            success: true,
            data: Some(student),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_students(state: State<'_, StudentServiceState>) -> Result<StudentListCommandResponse, String> {
    match state.list().await {
        Ok(students) => Ok(StudentListCommandResponse {
            success: true,
            data: Some(students),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_student(
    state: State<'_, StudentServiceState>,
    id: String,
    request: UpdateStudentCommand,
) -> Result<StudentCommandResponse, String> {
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
    ).await {
        Ok(student) => Ok(StudentCommandResponse {
            success: true,
            data: Some(student),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_student(state: State<'_, StudentServiceState>, id: String) -> Result<StudentCommandResponse, String> {
    match state.delete(&id).await {
        Ok(()) => Ok(StudentCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}
