//! Group Commands
//!
//! Tauri commands for group management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CreateGroupRequest, GroupDto, UpdateGroupRequest};
use crate::application::use_cases::GroupService;
use crate::infrastructure::repositories::{SqliteCourseRepository, SqliteGroupRepository};

pub type GroupServiceState = GroupService<SqliteGroupRepository, SqliteCourseRepository>;

/// Create group request payload
#[derive(Debug, Deserialize)]
pub struct CreateGroupCommand {
    pub course_id: String,
    pub name: String,
    pub professor_id: Option<String>,
    pub schedule: Option<String>,
    pub days: Option<Vec<String>>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub max_students: Option<i32>,
    pub class_duration: Option<i32>,
    pub skipped_dates: Option<Vec<String>>,
}

/// Update group request payload
#[derive(Debug, Deserialize)]
pub struct UpdateGroupCommand {
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

/// Group response payload
#[derive(Debug, Serialize)]
pub struct GroupCommandResponse {
    pub success: bool,
    pub data: Option<GroupDto>,
    pub error: Option<String>,
}

/// Group list response
#[derive(Debug, Serialize)]
pub struct GroupListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<GroupDto>>,
    pub error: Option<String>,
}

/// Create group command
#[tauri::command]
pub fn create_group(
    state: State<GroupServiceState>,
    request: CreateGroupCommand,
) -> GroupCommandResponse {
    match state.create(CreateGroupRequest {
        course_id: request.course_id,
        name: request.name,
        professor_id: request.professor_id,
        schedule: request.schedule,
        days: request.days,
        start_time: request.start_time,
        end_time: request.end_time,
        start_date: request.start_date,
        end_date: request.end_date,
        max_students: request.max_students.unwrap_or(20),
        class_duration: request.class_duration,
        skipped_dates: request.skipped_dates,
    }) {
        Ok(group) => GroupCommandResponse {
            success: true,
            data: Some(group),
            error: None,
        },
        Err(e) => GroupCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Get group by ID
#[tauri::command]
pub fn get_group(state: State<GroupServiceState>, id: String) -> GroupCommandResponse {
    match state.get_by_id(&id) {
        Ok(group) => GroupCommandResponse {
            success: true,
            data: Some(group),
            error: None,
        },
        Err(e) => GroupCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List all groups
#[tauri::command]
pub fn list_groups(state: State<GroupServiceState>) -> GroupListCommandResponse {
    match state.list() {
        Ok(groups) => GroupListCommandResponse {
            success: true,
            data: Some(groups),
            error: None,
        },
        Err(e) => GroupListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Update group
#[tauri::command]
pub fn update_group(
    state: State<GroupServiceState>,
    id: String,
    request: UpdateGroupCommand,
) -> GroupCommandResponse {
    match state.update(
        &id,
        UpdateGroupRequest {
            name: request.name,
            professor_id: request.professor_id,
            schedule: request.schedule,
            days: request.days,
            start_time: request.start_time,
            end_time: request.end_time,
            start_date: request.start_date,
            end_date: request.end_date,
            max_students: request.max_students,
            status: request.status,
            class_duration: request.class_duration,
            skipped_dates: request.skipped_dates,
        },
    ) {
        Ok(group) => GroupCommandResponse {
            success: true,
            data: Some(group),
            error: None,
        },
        Err(e) => GroupCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Delete group
#[tauri::command]
pub fn delete_group(state: State<GroupServiceState>, id: String) -> GroupCommandResponse {
    match state.delete(&id) {
        Ok(()) => GroupCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => GroupCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}
