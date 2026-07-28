use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CourseDto, CreateCourseRequest, UpdateCourseRequest};
use crate::application::use_cases::CourseService;
use crate::infrastructure::repositories::MemoryBackedCourseRepository;

pub type CourseServiceState = CourseService<MemoryBackedCourseRepository>;

#[derive(Debug, Deserialize)]
pub struct CreateCourseCommand {
    pub name: String,
    pub code: String,
    pub credits: i32,
    pub description: Option<String>,
    pub price: f64,
    pub duration: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCourseCommand {
    pub name: Option<String>,
    pub description: Option<String>,
    pub credits: Option<i32>,
    pub price: Option<f64>,
    pub duration: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CourseCommandResponse {
    pub success: bool,
    pub data: Option<CourseDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CourseListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<CourseDto>>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn create_course(
    state: State<'_, CourseServiceState>,
    request: CreateCourseCommand,
) -> Result<CourseCommandResponse, String> {
    match state.create(CreateCourseRequest {
        name: request.name,
        code: request.code,
        credits: request.credits,
        description: request.description,
        price: request.price,
        duration: request.duration,
    }).await {
        Ok(course) => Ok(CourseCommandResponse {
            success: true,
            data: Some(course),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_course(state: State<'_, CourseServiceState>, id: String) -> Result<CourseCommandResponse, String> {
    match state.get_by_id(&id).await {
        Ok(course) => Ok(CourseCommandResponse {
            success: true,
            data: Some(course),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_courses(state: State<'_, CourseServiceState>) -> Result<CourseListCommandResponse, String> {
    match state.list().await {
        Ok(courses) => Ok(CourseListCommandResponse {
            success: true,
            data: Some(courses),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_course(
    state: State<'_, CourseServiceState>,
    id: String,
    request: UpdateCourseCommand,
) -> Result<CourseCommandResponse, String> {
    match state.update(
        &id,
        UpdateCourseRequest {
            name: request.name,
            description: request.description,
            credits: request.credits,
            price: request.price,
            duration: request.duration,
        },
    ).await {
        Ok(course) => Ok(CourseCommandResponse {
            success: true,
            data: Some(course),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_course(state: State<'_, CourseServiceState>, id: String) -> Result<CourseCommandResponse, String> {
    match state.delete(&id).await {
        Ok(()) => Ok(CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn archive_course(state: State<'_, CourseServiceState>, id: String) -> Result<CourseCommandResponse, String> {
    match state.delete(&id).await {
        Ok(()) => Ok(CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn restore_course(state: State<'_, CourseServiceState>, id: String) -> Result<CourseCommandResponse, String> {
    match state.restore(&id).await {
        Ok(()) => Ok(CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn hard_delete_course(state: State<'_, CourseServiceState>, id: String) -> Result<CourseCommandResponse, String> {
    match state.hard_delete(&id).await {
        Ok(()) => Ok(CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_archived_courses(state: State<'_, CourseServiceState>) -> Result<CourseListCommandResponse, String> {
    match state.list_archived().await {
        Ok(courses) => Ok(CourseListCommandResponse {
            success: true,
            data: Some(courses),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}
