use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Information about an available update, serialized to the frontend.
#[derive(Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub release_notes: String,
    pub date: String,
    pub mandatory: bool,
}

/// Default config file name for updater settings.
const CONFIG_FILE: &str = "updater_config.json";

/// Default check interval in hours.
const DEFAULT_INTERVAL_HOURS: u64 = 4;

/// Check for an available update using tauri-plugin-updater.
/// Returns `Some(UpdateInfo)` if an update is available, `None` otherwise.
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    match update {
        Some(update) => {
            let info = UpdateInfo {
                version: update.version.clone(),
                release_notes: update.body.clone().unwrap_or_default(),
                date: update.date.map(|d| d.to_string()).unwrap_or_default(),
                mandatory: false,
            };
            Ok(Some(info))
        }
        None => Ok(None),
    }
}

/// Read the persisted update check interval (in hours) from the config file.
/// Defaults to 4 hours if the file does not exist or cannot be read.
#[tauri::command]
pub async fn get_update_check_interval() -> Result<u64, String> {
    let config_path = get_config_path();

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Ok(DEFAULT_INTERVAL_HOURS),
    };

    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let hours = json
        .get("check_interval_hours")
        .and_then(|v| v.as_u64())
        .unwrap_or(DEFAULT_INTERVAL_HOURS);

    Ok(hours)
}

/// Set the update check interval (in hours). Validates the value is within [1, 24]
/// and persists it to the config file.
#[tauri::command]
pub async fn set_update_check_interval(hours: u64) -> Result<(), String> {
    if hours < 1 || hours > 24 {
        return Err("Interval must be between 1 and 24 hours".to_string());
    }

    let config_path = get_config_path();

    // Ensure the parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::json!({ "check_interval_hours": hours });
    let content = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;

    std::fs::write(&config_path, content).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get the path to the updater config file in the app data directory.
fn get_config_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("academix")
        .join(CONFIG_FILE)
}
