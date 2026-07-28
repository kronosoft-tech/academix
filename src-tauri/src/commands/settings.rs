use serde::Serialize;
use tauri::State;

use crate::application::use_cases::SettingsService;
use crate::infrastructure::repositories::SqliteSettingsRepository;

pub type SettingsServiceState = SettingsService<SqliteSettingsRepository>;

#[derive(Debug, Serialize)]
pub struct ThresholdResponse {
    pub success: bool,
    pub data: Option<ThresholdDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ThresholdDto {
    pub value: i32,
}

#[tauri::command]
pub async fn get_absence_threshold(state: State<'_, SettingsServiceState>) -> Result<ThresholdResponse, String> {
    match state.get_absence_threshold().await {
        Ok(threshold) => Ok(ThresholdResponse {
            success: true,
            data: Some(ThresholdDto { value: threshold }),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn set_absence_threshold(
    state: State<'_, SettingsServiceState>,
    value: i32,
) -> Result<ThresholdResponse, String> {
    match state.set_absence_threshold(value).await {
        Ok(threshold) => Ok(ThresholdResponse {
            success: true,
            data: Some(ThresholdDto { value: threshold }),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}
