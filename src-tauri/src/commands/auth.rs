//! Authentication Commands
//!
//! Tauri commands for login/logout functionality via Turso.
//! Phase 4: Login resolves user via ControlPlane → ConnectionManager → libsql.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::application::dto::UserDto;
use crate::domain::entities::user::{Role, User};
use crate::env_loader;
use crate::infrastructure::password;
use crate::infrastructure::subscription_cache;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::control_plane::ControlPlaneRepository;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use std::collections::HashMap;

/// New AppState holding Turso infrastructure
pub struct AppState {
    pub connection_manager: Arc<Mutex<ConnectionManager>>,
    pub memory_buffer: Arc<Mutex<MemoryBuffer>>,
    pub control_plane: Option<Arc<ControlPlaneRepository>>,
    pub flush_timer_sender: Option<tokio::sync::oneshot::Sender<()>>,
    pub session: Arc<Mutex<CurrentSession>>,
    pub turso_config: Option<env_loader::TursoConfig>,
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
    let cp = state.control_plane.clone().ok_or_else(|| {
        "Turso not configured — set CONTROL_PLANE_DB_URL and CONTROL_PLANE_DB_TOKEN".to_string()
    })?;

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

    let user_id: String = user_row.get(0).map_err(|e| format!("Parse error: {}", e))?;
    let user_email: String = user_row.get(1).map_err(|e| format!("Parse error: {}", e))?;
    let password_hash: String = user_row.get(2).map_err(|e| format!("Parse error: {}", e))?;
    let user_name: String = user_row.get(3).map_err(|e| format!("Parse error: {}", e))?;
    let user_role: String = user_row.get(4).map_err(|e| format!("Parse error: {}", e))?;
    let is_active: i32 = user_row.get(5).map_err(|e| format!("Parse error: {}", e))?;

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
        let mut buffer = state.memory_buffer.lock().await;
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

    // Set the current session user_id (for MemoryBacked repo routing)
    {
        let mut sess = state.session.lock().await;
        sess.user_id = Some(user_id.clone());
        println!("[LOGIN] Session user_id set to '{}'", user_id);
    }

    // Step 9: Check subscription status
    let cache_path = subscription_cache::get_cache_path();
    let mapping_user_id = _mapping.user_id.clone();

    let sub_status = match cp.get_subscription_status(&mapping_user_id).await {
        Ok(Some((status, plan))) => {
            // Write to cache
            let _ = subscription_cache::write_cached_status(&cache_path, &status, plan.as_deref());
            status
        }
        Ok(None) => {
            // No subscription found — might be a legacy user
            "none".to_string()
        }
        Err(_) => {
            // Network error — check cache (24h grace)
            match subscription_cache::read_cached_status(&cache_path) {
                Some(cached) if subscription_cache::is_cache_valid(&cached.checked_at) => {
                    cached.status
                }
                _ => {
                    return Err(
                        "Cannot verify subscription status. Please check your internet connection."
                            .to_string(),
                    );
                }
            }
        }
    };

    // Block if expired or cancelled
    if sub_status == "expired" || sub_status == "cancelled" {
        return Err(format!(
            "Tu suscripción está {}. Visita https://academix.vercel.app/pricing para reactivar.",
            sub_status
        ));
    }

    // Step 10: Return login response
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
    // Resolve user_id from session for buffer routing
    let user_id = {
        let sess = state.session.lock().await;
        sess.user_id.clone().unwrap_or_else(|| "system".to_string())
    };

    // Buffer the session deletion — will be flushed to Turso
    let mut buffer = state.memory_buffer.lock().await;
    buffer.buffer_write(
        &user_id,
        BufferedOperation::Delete {
            table: "sessions".to_string(),
            id: request.token.clone(),
        },
    );
    drop(buffer);

    // Clear the session user_id
    {
        let mut sess = state.session.lock().await;
        sess.user_id = None;
    }

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
    state: State<'_, AppState>,
    request: CommandUpdateProfileRequest,
) -> Result<CommandUpdateProfileResponse, String> {
    let cp = state
        .control_plane
        .clone()
        .ok_or_else(|| "Turso not configured".to_string())?;

    let (user, db) = resolve_authenticated_user(
        &request.token,
        &cp,
        &state.connection_manager,
        &state.memory_buffer,
    )
    .await?;

    let conn = db
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE users SET name = ?1, email = ?2, updated_at = ?3 WHERE id = ?4",
        libsql::params![
            request.name.clone(),
            request.email.clone(),
            now,
            user.id.clone()
        ],
    )
    .await
    .map_err(|e| format!("Update failed: {}", e))?;

    // Also buffer the write via MemoryBuffer for consistency
    {
        let mut buffer = state.memory_buffer.lock().await;
        let mut data = HashMap::new();
        data.insert("name".to_string(), request.name.clone());
        data.insert("email".to_string(), request.email.clone());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        buffer.buffer_write(
            &user.id,
            BufferedOperation::Update {
                table: "users".to_string(),
                id: user.id.clone(),
                data,
            },
        );
    }

    Ok(CommandUpdateProfileResponse {
        success: true,
        user: Some(UserDto {
            id: user.id,
            email: request.email,
            name: request.name,
            role: user.role.as_str().to_string(),
        }),
        error: None,
    })
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
    state: State<'_, AppState>,
    request: CommandChangePasswordRequest,
) -> Result<CommandChangePasswordResponse, String> {
    let cp = state
        .control_plane
        .clone()
        .ok_or_else(|| "Turso not configured".to_string())?;

    let (user, db) = resolve_authenticated_user(
        &request.token,
        &cp,
        &state.connection_manager,
        &state.memory_buffer,
    )
    .await?;

    // Verify current password
    if !password::verify_password(&request.current_password, &user.password_hash) {
        return Ok(CommandChangePasswordResponse {
            success: false,
            error: Some("Current password is incorrect".to_string()),
        });
    }

    // Hash new password
    let new_hash = password::hash_password(&request.new_password).map_err(|e| e.to_string())?;

    let conn = db
        .connect()
        .map_err(|e| format!("Connection error: {}", e))?;

    conn.execute(
        "UPDATE users SET password_hash = ?1, updated_at = ?2 WHERE id = ?3",
        libsql::params![new_hash, Utc::now().to_rfc3339(), user.id],
    )
    .await
    .map_err(|e| format!("Update failed: {}", e))?;

    Ok(CommandChangePasswordResponse {
        success: true,
        error: None,
    })
}

/// Helper: convert a libsql row to a domain User entity.
fn user_from_row(row: libsql::Row) -> Result<User, String> {
    let id: String = row.get(0).map_err(|e| format!("Parse id: {}", e))?;
    let email: String = row.get(1).map_err(|e| format!("Parse email: {}", e))?;
    let password_hash: String = row
        .get(2)
        .map_err(|e| format!("Parse password_hash: {}", e))?;
    let name: String = row.get(3).map_err(|e| format!("Parse name: {}", e))?;
    let role_str: String = row.get(4).map_err(|e| format!("Parse role: {}", e))?;
    let is_active: i32 = row.get(5).map_err(|e| format!("Parse is_active: {}", e))?;
    let created_at_str: String = row.get(6).map_err(|e| format!("Parse created_at: {}", e))?;
    let updated_at_str: String = row.get(7).map_err(|e| format!("Parse updated_at: {}", e))?;

    let role = Role::from_str(&role_str).ok_or_else(|| format!("Invalid role: {}", role_str))?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| format!("Parse created_at: {}", e))?
        .with_timezone(&chrono::Utc);
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| format!("Parse updated_at: {}", e))?
        .with_timezone(&chrono::Utc);

    Ok(User {
        id,
        email,
        password_hash,
        name,
        role,
        active: is_active != 0,
        created_at,
        updated_at,
    })
}

/// Resolve an authenticated user from a session token.
///
/// Checks MemoryBuffer first (for sessions created in this session that haven't
/// been flushed yet), then iterates all cached Turso DB connections to find a
/// matching session record.
///
/// Phase 4: Linear scan of all connections. Phase 5 will add a proper
/// token→user_id index in MemoryBuffer for O(1) lookup.
pub async fn resolve_authenticated_user(
    token: &str,
    _control_plane: &ControlPlaneRepository,
    connection_manager: &Mutex<ConnectionManager>,
    memory_buffer: &Mutex<MemoryBuffer>,
) -> Result<(User, Arc<libsql::Database>), String> {
    // Step 1: Check MemoryBuffer for recently created (not yet flushed) sessions
    {
        let buffer = memory_buffer.lock().await;
        if let Some((user_id, _session_data)) = buffer.find_session_by_token(token) {
            // Found in pending writes — user recently logged in.
            // Directly look up the user's cached connection.
            let cm = connection_manager.lock().await;
            if let Some(cached) = cm.get_connection(&user_id) {
                let conn = cached
                    .db
                    .connect()
                    .map_err(|e| format!("Connection error: {}", e))?;

                let mut rows = conn
                    .query(
                        "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at FROM users WHERE id = ?1",
                        libsql::params![user_id],
                    )
                    .await
                    .map_err(|e| format!("Query error: {}", e))?;

                if let Some(row) = rows.next().await.map_err(|e| format!("Row error: {}", e))? {
                    let user = user_from_row(row)?;
                    return Ok((user, Arc::clone(&cached.db)));
                }
            }
        }
    }

    // Step 2: Iterate all cached connections to find a matching session
    let connections = {
        let cm = connection_manager.lock().await;
        cm.get_all_connections()
    };

    for cached in &connections {
        let libsql_conn = cached
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;

        // Query sessions table for this token (only valid, non-expired sessions)
        let mut rows = libsql_conn
            .query(
                "SELECT user_id FROM sessions WHERE token = ?1 AND expires_at > datetime('now')",
                libsql::params![token],
            )
            .await
            .map_err(|e| format!("Session query error: {}", e))?;

        if let Some(row) = rows.next().await.map_err(|e| format!("Row error: {}", e))? {
            let user_id: String = row.get(0).map_err(|e| format!("Parse user_id: {}", e))?;

            // Found the session — now get the user
            let mut user_rows = libsql_conn
                .query(
                    "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at FROM users WHERE id = ?1",
                    libsql::params![user_id],
                )
                .await
                .map_err(|e| format!("User query error: {}", e))?;

            if let Some(user_row) = user_rows
                .next()
                .await
                .map_err(|e| format!("Row error: {}", e))?
            {
                let user = user_from_row(user_row)?;
                return Ok((user, Arc::clone(&cached.db)));
            }
        }
    }

    Err("Session not found or expired".to_string())
}
