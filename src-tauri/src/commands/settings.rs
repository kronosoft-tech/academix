//! Settings Commands
//!
//! Tauri commands for application settings management.

use serde::Serialize;
use tauri::State;

use crate::application::use_cases::SettingsService;
use crate::infrastructure::repositories::SqliteSettingsRepository;

pub type SettingsServiceState = SettingsService<SqliteSettingsRepository>;

/// Threshold response payload
#[derive(Debug, Serialize)]
pub struct ThresholdResponse {
    pub success: bool,
    pub data: Option<ThresholdDto>,
    pub error: Option<String>,
}

/// Threshold DTO
#[derive(Debug, Serialize)]
pub struct ThresholdDto {
    pub value: i32,
}

/// Get absence threshold
#[tauri::command]
pub fn get_absence_threshold(state: State<SettingsServiceState>) -> ThresholdResponse {
    match state.get_absence_threshold() {
        Ok(threshold) => ThresholdResponse {
            success: true,
            data: Some(ThresholdDto { value: threshold }),
            error: None,
        },
        Err(e) => ThresholdResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Set absence threshold
#[tauri::command]
pub fn set_absence_threshold(
    state: State<SettingsServiceState>,
    value: i32,
) -> ThresholdResponse {
    match state.set_absence_threshold(value) {
        Ok(threshold) => ThresholdResponse {
            success: true,
            data: Some(ThresholdDto { value: threshold }),
            error: None,
        },
        Err(e) => ThresholdResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}