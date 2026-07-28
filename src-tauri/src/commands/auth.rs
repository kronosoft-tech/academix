//! Authentication Commands
//!
//! Tauri commands for login/logout functionality via Turso.
//! Phase 4: Login resolves user via ControlPlane → ConnectionManager → libsql.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::application::dto::UserDto;
use crate::infrastructure::password;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::control_plane::ControlPlaneRepository;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// New AppState holding Turso infrastructure
pub struct AppState {
    pub connection_manager: Arc<Mutex<ConnectionManager>>,
    pub memory_buffer: Arc<Mutex<MemoryBuffer>>,
    pub control_plane: Option<Arc<ControlPlaneRepository>>,
    pub flush_timer_sender: Option<tokio::sync::oneshot::Sender<()>>,
}

/// Login request payload
#[derive(Debug, Deserialize)]
pub struct CommandLoginRequest {
    pub email: String,
    pub password: String,
}

/// Login response payload
#[derive(Debug, Serialize)]
pub struct CommandLoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<UserDto>,
    pub expires_at: Option<String>,
    pub error: Option<String>,
}

/// Login command — fully async via Turso
#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    request: CommandLoginRequest,
) -> Result<CommandLoginResponse, String> {
    let email = request.email.clone();
    let password = request.password.clone();

    // Step 1: Get control plane reference
    let cp = state
        .control_plane
        .clone()
        .ok_or_else(|| "Turso not configured — set CONTROL_PLANE_DB_URL and CONTROL_PLANE_DB_TOKEN".to_string())?;

    // Step 2: Look up user's Turso DB mapping via control plane
    let _mapping = cp
        .find_by_email(&email)
        .await
        .map_err(|e| format!("Control plane error: {}", e))?
        .ok_or_else(|| "Invalid credentials".to_string())?;

    // Step 3: Resolve user's Turso DB connection (lock dropped after clone)
    let cached = {
        let mut cm = state.connection_manager.lock().await;
        cm.resolve_by_email(&cp, &email)
            .await
            .map_err(|e| format!("DB resolution error: {}", e))?
            .clone()
    };

    // Step 4: Connect to user's Turso DB and query users table
    let libsql_conn = cached
        .db
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    // Step 5: Query user by email
    let mut rows = libsql_conn
        .query(
            "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at FROM users WHERE email = ?1",
            libsql::params![email.clone()],
        )
        .await
        .map_err(|e| format!("Query error: {}", e))?;

    let user_row = rows
        .next()
        .await
        .map_err(|e| format!("Row fetch error: {}", e))?
        .ok_or_else(|| "Invalid credentials".to_string())?;

    let user_id: String = user_row
        .get(0)
        .map_err(|e| format!("Parse error: {}", e))?;
    let user_email: String = user_row
        .get(1)
        .map_err(|e| format!("Parse error: {}", e))?;
    let password_hash: String = user_row
        .get(2)
        .map_err(|e| format!("Parse error: {}", e))?;
    let user_name: String = user_row
        .get(3)
        .map_err(|e| format!("Parse error: {}", e))?;
    let user_role: String = user_row
        .get(4)
        .map_err(|e| format!("Parse error: {}", e))?;
    let is_active: i32 = user_row
        .get(5)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Step 6: Verify password
    if !password::verify_password(&password, &password_hash) {
        return Err("Invalid credentials".to_string());
    }

    // Step 7: Check if user is active
    if is_active == 0 {
        return Err("User account is inactive".to_string());
    }

    // Step 8: Create session
    let session_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();
    let token = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();
    let expires_at = Utc::now() + Duration::hours(24);
    let expires_at_str = expires_at.to_rfc3339();

    // Buffer session write in MemoryBuffer
    {
        let mut buffer = state
            .memory_buffer
            .lock()
            .await;
        let mut session_data = HashMap::new();
        session_data.insert("id".to_string(), session_id.clone());
        session_data.insert("user_id".to_string(), user_id.clone());
        session_data.insert("token".to_string(), token.clone());
        session_data.insert("expires_at".to_string(), expires_at_str.clone());
        session_data.insert("created_at".to_string(), Utc::now().to_rfc3339());

        buffer.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "sessions".to_string(),
                data: session_data,
            },
        );
    }

    // Step 9: Return login response
    Ok(CommandLoginResponse {
        success: true,
        token: Some(token),
        user: Some(UserDto {
            id: user_id,
            email: user_email,
            name: user_name,
            role: user_role.to_lowercase(),
        }),
        expires_at: Some(expires_at_str),
        error: None,
    })
}

/// Logout request payload
#[derive(Debug, Deserialize)]
pub struct CommandLogoutRequest {
    pub token: String,
}

/// Logout response payload
#[derive(Debug, Serialize)]
pub struct CommandLogoutResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Logout command — buffers session deletion via MemoryBuffer
#[tauri::command]
pub async fn logout(
    state: State<'_, AppState>,
    request: CommandLogoutRequest,
) -> Result<CommandLogoutResponse, String> {
    // Buffer the session deletion — will be flushed to Turso
    let mut buffer = state.memory_buffer.lock().await;
    buffer.buffer_write(
        "system",
        BufferedOperation::Delete {
            table: "sessions".to_string(),
            id: request.token.clone(),
        },
    );
    drop(buffer);

    Ok(CommandLogoutResponse {
        success: true,
        error: None,
    })
}

/// Update profile request payload
#[derive(Debug, Deserialize)]
pub struct CommandUpdateProfileRequest {
    pub token: String,
    pub name: String,
    pub email: String,
}

/// Update profile response payload
#[derive(Debug, Serialize)]
pub struct CommandUpdateProfileResponse {
    pub success: bool,
    pub user: Option<UserDto>,
    pub error: Option<String>,
}

/// Update own profile (name and email only)
#[tauri::command]
pub async fn update_profile(
    _state: State<'_, AppState>,
    _request: CommandUpdateProfileRequest,
) -> Result<CommandUpdateProfileResponse, String> {
    // Phase 4: Stub implementation — resolves the user and buffers the update
    // Full implementation requires resolve_authenticated_user to be completed
    // For now, this is a placeholder that returns an error with guidance
    Err(
        "Profile update via Turso not yet implemented — Phase 5 will complete this"
            .to_string(),
    )
}

/// Change password request payload
#[derive(Debug, Deserialize)]
pub struct CommandChangePasswordRequest {
    pub token: String,
    pub current_password: String,
    pub new_password: String,
}

/// Change password response payload
#[derive(Debug, Serialize)]
pub struct CommandChangePasswordResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Change own password (requires current password verification)
#[tauri::command]
pub async fn change_password(
    _state: State<'_, AppState>,
    _request: CommandChangePasswordRequest,
) -> Result<CommandChangePasswordResponse, String> {
    // Phase 4: Stub implementation
    // Full implementation requires resolve_authenticated_user
    Err(
        "Password change via Turso not yet implemented — Phase 5 will complete this"
            .to_string(),
    )
}

/// Resolve an authenticated user from a session token.
///
/// Checks MemoryBuffer first, then the user's Turso DB.
/// Phase 4: Basic implementation that searches for the session via the user's
/// Turso DB. Phase 5 will add MemoryBuffer-first lookup.
pub async fn resolve_authenticated_user(
    token: &str,
    control_plane: &ControlPlaneRepository,
    connection_manager: &Mutex<ConnectionManager>,
    memory_buffer: &Mutex<MemoryBuffer>,
) -> Result<(crate::domain::entities::user::User, libsql::Database), String> {
    // Phase 4 simplification: iterate cached connections to find the session
    // In Phase 5, this will be optimized with a token→user_id index in MemoryBuffer

    // Get all connected user IDs
    // For now, return a helpful error — this will be completed when commands migrate
    let _ = token;
    let _ = control_plane;
    let _ = connection_manager;
    let _ = memory_buffer;

    Err(
        "resolve_authenticated_user not fully implemented — will be completed when commands migrate"
            .to_string(),
    )
}
