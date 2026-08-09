//! Update Scheduler Infrastructure
//!
//! Background task that periodically checks for application updates
//! using `tauri-plugin-updater` and emits events to the frontend
//! when a new version is available.

use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;
use tokio::time::{sleep, Duration};

/// Payload emitted to the frontend when an update is available.
#[derive(serde::Serialize, Clone)]
pub struct UpdateAvailablePayload {
    pub version: String,
    pub release_notes: String,
    pub date: String,
    pub mandatory: bool,
}

/// Manages periodic update checks in the background.
pub struct UpdateScheduler;

impl UpdateScheduler {
    /// Spawns a background task that checks for updates periodically.
    ///
    /// - Performs an initial check 10 seconds after being started.
    /// - Repeats at the configured `interval_hours` (default: 4).
    /// - Emits `update-available` event to the frontend when a new version is found.
    /// - Logs failures silently without disrupting the application.
    pub fn start(app_handle: AppHandle, interval_hours: u64) {
        tauri::async_runtime::spawn(async move {
            // Initial delay: 10 seconds after app ready
            sleep(Duration::from_secs(10)).await;

            loop {
                match Self::perform_check(&app_handle).await {
                    Ok(true) => {
                        eprintln!("[UPDATER] Update available — event emitted to frontend");
                    }
                    Ok(false) => {
                        eprintln!("[UPDATER] No update available");
                    }
                    Err(e) => {
                        eprintln!("[UPDATER] Check failed: {}", e);
                    }
                }

                // Wait for next interval
                sleep(Duration::from_secs(interval_hours * 3600)).await;
            }
        });
    }

    /// Performs a single update check using the updater plugin.
    ///
    /// Returns `Ok(true)` if an update is available and the event was emitted,
    /// `Ok(false)` if no update is available, or `Err` on failure.
    async fn perform_check(app_handle: &AppHandle) -> Result<bool, String> {
        let updater = app_handle
            .updater()
            .map_err(|e| format!("Failed to get updater: {}", e))?;

        let update = updater
            .check()
            .await
            .map_err(|e| format!("Update check failed: {}", e))?;

        match update {
            Some(update) => {
                let payload = UpdateAvailablePayload {
                    version: update.version.clone(),
                    release_notes: update.body.clone().unwrap_or_default(),
                    date: update.date.map(|d| d.to_string()).unwrap_or_default(),
                    mandatory: false,
                };

                app_handle
                    .emit("update-available", payload)
                    .map_err(|e| format!("Failed to emit event: {}", e))?;

                Ok(true)
            }
            None => Ok(false),
        }
    }
}
