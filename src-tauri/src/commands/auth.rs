//! Authentication Commands
//!
//! Tauri commands for login/logout functionality.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{LoginRequest, UserDto};
use crate::application::use_cases::AuthService;
use crate::infrastructure::database::SqlitePool;
use crate::infrastructure::repositories::{InMemorySessionRepository, SqliteUserRepository};
use std::sync::Arc;

/// Application state holding services
pub struct AppState {
    pub auth_service: AuthService<SqliteUserRepository, InMemorySessionRepository>,
}

impl AppState {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        let user_repo = SqliteUserRepository::new(pool);
        let session_repo = InMemorySessionRepository::new();

        Self {
            auth_service: AuthService::new(user_repo, session_repo),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        // This won't be called in practice since we pass the pool from lib.rs
        Self::new(Arc::new(
            SqlitePool::new(std::path::PathBuf::from("dummy.db")).unwrap(),
        ))
    }
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

/// Login command
#[tauri::command]
pub fn login(state: State<AppState>, request: CommandLoginRequest) -> CommandLoginResponse {
    let email = request.email.clone();
    println!("[DEBUG] login called with email: {}", email);
    match state.auth_service.login(LoginRequest {
        email: request.email,
        password: request.password,
    }) {
        Ok(response) => {
            println!("[DEBUG] login success for: {}", email);
            CommandLoginResponse {
                success: true,
                token: Some(response.token),
                user: Some(response.user),
                expires_at: Some(response.expires_at),
                error: None,
            }
        }
        Err(e) => {
            println!("[DEBUG] login error: {} for email: {}", e, email);
            CommandLoginResponse {
                success: false,
                token: None,
                user: None,
                expires_at: None,
                error: Some(e.to_string()),
            }
        }
    }
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

/// Logout command
#[tauri::command]
pub fn logout(state: State<AppState>, request: CommandLogoutRequest) -> CommandLogoutResponse {
    match state.auth_service.logout(&request.token) {
        Ok(()) => CommandLogoutResponse {
            success: true,
            error: None,
        },
        Err(e) => CommandLogoutResponse {
            success: false,
            error: Some(e.to_string()),
        },
    }
}
