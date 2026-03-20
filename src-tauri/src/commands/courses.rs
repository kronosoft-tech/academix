//! Course Commands
//!
//! Tauri commands for course management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CourseDto, CreateCourseRequest, UpdateCourseRequest};
use crate::application::use_cases::CourseService;
use crate::infrastructure::repositories::SqliteCourseRepository;

pub type CourseServiceState = CourseService<SqliteCourseRepository>;

/// Create course request payload
#[derive(Debug, Deserialize)]
pub struct CreateCourseCommand {
    pub name: String,
    pub code: String,
    pub credits: i32,
    pub description: Option<String>,
    pub price: f64,
    pub duration: i32,
}

/// Update course request payload
#[derive(Debug, Deserialize)]
pub struct UpdateCourseCommand {
    pub name: Option<String>,
    pub description: Option<String>,
    pub credits: Option<i32>,
    pub price: Option<f64>,
    pub duration: Option<i32>,
}

/// Course response payload
#[derive(Debug, Serialize)]
pub struct CourseCommandResponse {
    pub success: bool,
    pub data: Option<CourseDto>,
    pub error: Option<String>,
}

/// Course list response
#[derive(Debug, Serialize)]
pub struct CourseListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<CourseDto>>,
    pub error: Option<String>,
}

/// Create course command
#[tauri::command]
pub fn create_course(
    state: State<CourseServiceState>,
    request: CreateCourseCommand,
) -> CourseCommandResponse {
    println!("[DEBUG] create_course called: {:?}", request);
    match state.create(CreateCourseRequest {
        name: request.name,
        code: request.code,
        credits: request.credits,
        description: request.description,
        price: request.price,
        duration: request.duration,
    }) {
        Ok(course) => {
            println!("[DEBUG] create_course success: {:?}", course);
            CourseCommandResponse {
                success: true,
                data: Some(course),
                error: None,
            }
        }
        Err(e) => {
            println!("[DEBUG] create_course error: {}", e);
            CourseCommandResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Get course by ID
#[tauri::command]
pub fn get_course(state: State<CourseServiceState>, id: String) -> CourseCommandResponse {
    match state.get_by_id(&id) {
        Ok(course) => CourseCommandResponse {
            success: true,
            data: Some(course),
            error: None,
        },
        Err(e) => CourseCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List all courses
#[tauri::command]
pub fn list_courses(state: State<CourseServiceState>) -> CourseListCommandResponse {
    println!("[DEBUG] list_courses called");
    match state.list() {
        Ok(courses) => {
            println!("[DEBUG] list_courses returned {} courses", courses.len());
            CourseListCommandResponse {
                success: true,
                data: Some(courses),
                error: None,
            }
        }
        Err(e) => {
            println!("[DEBUG] list_courses error: {}", e);
            CourseListCommandResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Update course
#[tauri::command]
pub fn update_course(
    state: State<CourseServiceState>,
    id: String,
    request: UpdateCourseCommand,
) -> CourseCommandResponse {
    match state.update(
        &id,
        UpdateCourseRequest {
            name: request.name,
            description: request.description,
            credits: request.credits,
            price: request.price,
            duration: request.duration,
        },
    ) {
        Ok(course) => CourseCommandResponse {
            success: true,
            data: Some(course),
            error: None,
        },
        Err(e) => CourseCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Delete course
#[tauri::command]
pub fn delete_course(state: State<CourseServiceState>, id: String) -> CourseCommandResponse {
    match state.delete(&id) {
        Ok(()) => CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => CourseCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Archive course (soft delete - marks as archived)
#[tauri::command]
pub fn archive_course(state: State<CourseServiceState>, id: String) -> CourseCommandResponse {
    match state.delete(&id) {
        Ok(()) => CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => CourseCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Restore archived course
#[tauri::command]
pub fn restore_course(state: State<CourseServiceState>, id: String) -> CourseCommandResponse {
    match state.restore(&id) {
        Ok(()) => CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => CourseCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Hard delete course (permanently removes from database)
#[tauri::command]
pub fn hard_delete_course(state: State<CourseServiceState>, id: String) -> CourseCommandResponse {
    match state.hard_delete(&id) {
        Ok(()) => CourseCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => CourseCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List archived courses
#[tauri::command]
pub fn list_archived_courses(state: State<CourseServiceState>) -> CourseListCommandResponse {
    match state.list_archived() {
        Ok(courses) => CourseListCommandResponse {
            success: true,
            data: Some(courses),
            error: None,
        },
        Err(e) => CourseListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}
