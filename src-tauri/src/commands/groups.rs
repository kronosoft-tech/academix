use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CreateGroupRequest, GroupDto, UpdateGroupRequest};
use crate::application::use_cases::GroupService;
use crate::infrastructure::repositories::{MemoryBackedCourseRepository, MemoryBackedGroupRepository};

pub type GroupServiceState = GroupService<MemoryBackedGroupRepository, MemoryBackedCourseRepository>;

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

#[derive(Debug, Serialize)]
pub struct GroupCommandResponse {
    pub success: bool,
    pub data: Option<GroupDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<GroupDto>>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn create_group(
    state: State<'_, GroupServiceState>,
    request: CreateGroupCommand,
) -> Result<GroupCommandResponse, String> {
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
    }).await {
        Ok(group) => Ok(GroupCommandResponse {
            success: true,
            data: Some(group),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_group(state: State<'_, GroupServiceState>, id: String) -> Result<GroupCommandResponse, String> {
    match state.get_by_id(&id).await {
        Ok(group) => Ok(GroupCommandResponse {
            success: true,
            data: Some(group),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_groups(state: State<'_, GroupServiceState>) -> Result<GroupListCommandResponse, String> {
    match state.list().await {
        Ok(groups) => Ok(GroupListCommandResponse {
            success: true,
            data: Some(groups),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_group(
    state: State<'_, GroupServiceState>,
    id: String,
    request: UpdateGroupCommand,
) -> Result<GroupCommandResponse, String> {
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
    ).await {
        Ok(group) => Ok(GroupCommandResponse {
            success: true,
            data: Some(group),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_group(state: State<'_, GroupServiceState>, id: String) -> Result<GroupCommandResponse, String> {
    match state.delete(&id).await {
        Ok(()) => Ok(GroupCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}
