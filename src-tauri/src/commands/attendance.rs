use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{
    AttendanceDto, CreateAttendanceRequest, GroupAttendanceStats, StudentAbsenceCountDto,
    UpdateAttendanceRequest,
};
use crate::application::use_cases::AttendanceService;
use crate::infrastructure::repositories::SqliteAttendanceRepository;

pub type AttendanceServiceState = AttendanceService<SqliteAttendanceRepository>;

#[derive(Debug, Deserialize)]
pub struct CreateAttendanceCommand {
    pub student_id: String,
    pub group_id: String,
    pub date: String,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAttendanceCommand {
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AttendanceCommandResponse {
    pub success: bool,
    pub data: Option<AttendanceDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AttendanceListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<AttendanceDto>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupAttendanceStatsResponse {
    pub success: bool,
    pub data: Option<GroupAttendanceStats>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AbsenceCountResponse {
    pub success: bool,
    pub data: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupAbsenceCountsResponse {
    pub success: bool,
    pub data: Option<Vec<StudentAbsenceCountDto>>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn create_attendance(
    state: State<'_, AttendanceServiceState>,
    request: CreateAttendanceCommand,
) -> Result<AttendanceCommandResponse, String> {
    match state.create(CreateAttendanceRequest {
        student_id: request.student_id,
        group_id: request.group_id,
        date: request.date,
        status: request.status,
        notes: request.notes,
    }).await {
        Ok(attendance) => Ok(AttendanceCommandResponse {
            success: true,
            data: Some(attendance),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_attendance(
    state: State<'_, AttendanceServiceState>,
    id: String,
) -> Result<AttendanceCommandResponse, String> {
    match state.get_by_id(&id).await {
        Ok(attendance) => Ok(AttendanceCommandResponse {
            success: true,
            data: Some(attendance),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_attendances(state: State<'_, AttendanceServiceState>) -> Result<AttendanceListCommandResponse, String> {
    match state.list().await {
        Ok(attendances) => Ok(AttendanceListCommandResponse {
            success: true,
            data: Some(attendances),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_attendance_by_group_date(
    state: State<'_, AttendanceServiceState>,
    group_id: String,
    date: String,
) -> Result<AttendanceListCommandResponse, String> {
    match state.list_by_group_and_date(&group_id, &date).await {
        Ok(attendances) => Ok(AttendanceListCommandResponse {
            success: true,
            data: Some(attendances),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_attendance_by_student(
    state: State<'_, AttendanceServiceState>,
    student_id: String,
) -> Result<AttendanceListCommandResponse, String> {
    match state.list_by_student(&student_id).await {
        Ok(attendances) => Ok(AttendanceListCommandResponse {
            success: true,
            data: Some(attendances),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_attendance(
    state: State<'_, AttendanceServiceState>,
    id: String,
    request: UpdateAttendanceCommand,
) -> Result<AttendanceCommandResponse, String> {
    match state.update(
        &id,
        UpdateAttendanceRequest {
            status: request.status,
            notes: request.notes,
        },
    ).await {
        Ok(attendance) => Ok(AttendanceCommandResponse {
            success: true,
            data: Some(attendance),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_attendance(
    state: State<'_, AttendanceServiceState>,
    id: String,
) -> Result<AttendanceCommandResponse, String> {
    match state.delete(&id).await {
        Ok(()) => Ok(AttendanceCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_group_attendance_stats(
    state: State<'_, AttendanceServiceState>,
    group_id: String,
    total_students: i32,
) -> Result<GroupAttendanceStatsResponse, String> {
    match state.get_group_stats(&group_id, total_students).await {
        Ok(stats) => Ok(GroupAttendanceStatsResponse {
            success: true,
            data: Some(stats),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn count_student_absences(
    state: State<'_, AttendanceServiceState>,
    student_id: String,
    group_id: String,
) -> Result<AbsenceCountResponse, String> {
    match state.count_student_absences(&student_id, &group_id).await {
        Ok(count) => Ok(AbsenceCountResponse {
            success: true,
            data: Some(count),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn count_group_absences(
    state: State<'_, AttendanceServiceState>,
    group_id: String,
) -> Result<GroupAbsenceCountsResponse, String> {
    match state.count_group_absences(&group_id).await {
        Ok(counts) => Ok(GroupAbsenceCountsResponse {
            success: true,
            data: Some(counts),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}
