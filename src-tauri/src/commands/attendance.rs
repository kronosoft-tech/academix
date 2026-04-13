//! Attendance Commands
//!
//! Tauri commands for attendance management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{
    AttendanceDto, CreateAttendanceRequest, GroupAttendanceStats, UpdateAttendanceRequest,
};
use crate::application::use_cases::AttendanceService;
use crate::infrastructure::repositories::SqliteAttendanceRepository;

pub type AttendanceServiceState = AttendanceService<SqliteAttendanceRepository>;

/// Create attendance request payload
#[derive(Debug, Deserialize)]
pub struct CreateAttendanceCommand {
    pub student_id: String,
    pub group_id: String,
    pub date: String,
    pub status: String,
    pub notes: Option<String>,
}

/// Update attendance request payload
#[derive(Debug, Deserialize)]
pub struct UpdateAttendanceCommand {
    pub status: Option<String>,
    pub notes: Option<String>,
}

/// Attendance response payload
#[derive(Debug, Serialize)]
pub struct AttendanceCommandResponse {
    pub success: bool,
    pub data: Option<AttendanceDto>,
    pub error: Option<String>,
}

/// Attendance list response
#[derive(Debug, Serialize)]
pub struct AttendanceListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<AttendanceDto>>,
    pub error: Option<String>,
}

/// Group attendance stats response
#[derive(Debug, Serialize)]
pub struct GroupAttendanceStatsResponse {
    pub success: bool,
    pub data: Option<GroupAttendanceStats>,
    pub error: Option<String>,
}

/// Create attendance command
#[tauri::command]
pub fn create_attendance(
    state: State<AttendanceServiceState>,
    request: CreateAttendanceCommand,
) -> AttendanceCommandResponse {
    match state.create(CreateAttendanceRequest {
        student_id: request.student_id,
        group_id: request.group_id,
        date: request.date,
        status: request.status,
        notes: request.notes,
    }) {
        Ok(attendance) => AttendanceCommandResponse {
            success: true,
            data: Some(attendance),
            error: None,
        },
        Err(e) => AttendanceCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Get attendance by ID
#[tauri::command]
pub fn get_attendance(
    state: State<AttendanceServiceState>,
    id: String,
) -> AttendanceCommandResponse {
    match state.get_by_id(&id) {
        Ok(attendance) => AttendanceCommandResponse {
            success: true,
            data: Some(attendance),
            error: None,
        },
        Err(e) => AttendanceCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List all attendance records
#[tauri::command]
pub fn list_attendances(state: State<AttendanceServiceState>) -> AttendanceListCommandResponse {
    println!("[DEBUG] list_attendances called");
    match state.list() {
        Ok(attendances) => {
            println!(
                "[DEBUG] list_attendances returned {} records",
                attendances.len()
            );
            AttendanceListCommandResponse {
                success: true,
                data: Some(attendances),
                error: None,
            }
        }
        Err(e) => {
            println!("[DEBUG] list_attendances error: {}", e);
            AttendanceListCommandResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// List attendance by group and date
#[tauri::command]
pub fn list_attendance_by_group_date(
    state: State<AttendanceServiceState>,
    group_id: String,
    date: String,
) -> AttendanceListCommandResponse {
    match state.list_by_group_and_date(&group_id, &date) {
        Ok(attendances) => AttendanceListCommandResponse {
            success: true,
            data: Some(attendances),
            error: None,
        },
        Err(e) => AttendanceListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List attendance by student
#[tauri::command]
pub fn list_attendance_by_student(
    state: State<AttendanceServiceState>,
    student_id: String,
) -> AttendanceListCommandResponse {
    match state.list_by_student(&student_id) {
        Ok(attendances) => AttendanceListCommandResponse {
            success: true,
            data: Some(attendances),
            error: None,
        },
        Err(e) => AttendanceListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Update attendance
#[tauri::command]
pub fn update_attendance(
    state: State<AttendanceServiceState>,
    id: String,
    request: UpdateAttendanceCommand,
) -> AttendanceCommandResponse {
    match state.update(
        &id,
        UpdateAttendanceRequest {
            status: request.status,
            notes: request.notes,
        },
    ) {
        Ok(attendance) => AttendanceCommandResponse {
            success: true,
            data: Some(attendance),
            error: None,
        },
        Err(e) => AttendanceCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Delete attendance
#[tauri::command]
pub fn delete_attendance(
    state: State<AttendanceServiceState>,
    id: String,
) -> AttendanceCommandResponse {
    match state.delete(&id) {
        Ok(()) => AttendanceCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => AttendanceCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Get group attendance statistics
#[tauri::command]
pub fn get_group_attendance_stats(
    state: State<AttendanceServiceState>,
    group_id: String,
    total_students: i32,
) -> GroupAttendanceStatsResponse {
    match state.get_group_stats(&group_id, total_students) {
        Ok(stats) => GroupAttendanceStatsResponse {
            success: true,
            data: Some(stats),
            error: None,
        },
        Err(e) => GroupAttendanceStatsResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}
