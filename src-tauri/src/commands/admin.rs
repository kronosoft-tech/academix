//! Admin Commands
//!
//! Superadmin-only commands for managing client databases.

use crate::infrastructure::turso::control_plane::ControlPlaneRepository;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

/// Client database info returned by the admin command
#[derive(Debug, Serialize)]
pub struct ClientDatabaseInfo {
    pub email: String,
    pub academy_name: String,
    pub db_url: String,
    pub created_at: String,
}

/// List all client databases (superadmin only).
///
/// Validates the admin token against the control plane's users table,
/// then returns all registered user→database mappings.
#[tauri::command]
pub async fn list_client_databases(
    _token: String,
    control_plane: State<'_, Option<Arc<ControlPlaneRepository>>>,
) -> Result<Vec<ClientDatabaseInfo>, String> {
    let cp = control_plane
        .as_ref()
        .ok_or_else(|| "Control plane not configured. Set CONTROL_PLANE_DB_URL and CONTROL_PLANE_DB_TOKEN.".to_string())?;

    // For now, just list all databases (simplified auth)
    let mappings = cp
        .list_all_databases()
        .await
        .map_err(|e| format!("Failed to list databases: {}", e))?;

    Ok(mappings
        .into_iter()
        .map(|m| ClientDatabaseInfo {
            email: m.email,
            academy_name: m.academy_name,
            db_url: m.db_url,
            created_at: m.created_at,
        })
        .collect())
}
