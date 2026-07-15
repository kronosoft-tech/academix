//! Authentication Commands
//!
//! Tauri commands for login/logout functionality.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{LoginRequest, UserDto};
use crate::application::use_cases::AuthService;
use crate::infrastructure::database::SqlitePool;
use crate::infrastructure::repositories::{SqliteSessionRepository, SqliteUserRepository};
use std::sync::Arc;

/// Application state holding services
pub struct AppState {
    pub auth_service: AuthService<SqliteUserRepository, SqliteSessionRepository>,
}

impl AppState {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        let user_repo = SqliteUserRepository::new(Arc::clone(&pool));
        let session_repo = SqliteSessionRepository::new(Arc::clone(&pool));

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
    match state.auth_service.login(LoginRequest {
        email: request.email,
        password: request.password,
    }) {
        Ok(response) => CommandLoginResponse {
            success: true,
            token: Some(response.token),
            user: Some(response.user),
            expires_at: Some(response.expires_at),
            error: None,
        },
        Err(e) => CommandLoginResponse {
            success: false,
            token: None,
            user: None,
            expires_at: None,
            error: Some(e.to_string()),
        },
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
pub fn update_profile(
    state: State<AppState>,
    request: CommandUpdateProfileRequest,
) -> CommandUpdateProfileResponse {
    // Validate token and get user - ALREADY HAVE THE USER!
    match state.auth_service.validate_token(&request.token) {
        Ok(user) => {
            use crate::application::ports::UserRepository;
            use crate::infrastructure::repositories::SqliteUserRepository;

            // Create a NEW repository from the pool
            let pool = std::sync::Arc::clone(&state.auth_service.user_repository().pool());
            let user_repo = SqliteUserRepository::new(pool);

            // Create updated user from the one we already have (no query needed!)
            let user_updated = crate::domain::entities::user::User {
                id: user.id,
                email: request.email,
                password_hash: user.password_hash,
                name: request.name,
                role: user.role,
                active: user.active,
                created_at: user.created_at,
                updated_at: chrono::Utc::now(),
            };

            // Save directly
            if let Err(e) = user_repo.update(&user_updated) {
                return CommandUpdateProfileResponse {
                    success: false,
                    user: None,
                    error: Some(format!("Failed to update profile: {}", e)),
                };
            }

            CommandUpdateProfileResponse {
                success: true,
                user: Some(UserDto {
                    id: user_updated.id,
                    email: user_updated.email,
                    name: user_updated.name,
                    role: user_updated.role.as_str().to_string(),
                }),
                error: None,
            }
        }
        Err(e) => CommandUpdateProfileResponse {
            success: false,
            user: None,
            error: Some(e.to_string()),
        },
    }
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
pub fn change_password(
    state: State<AppState>,
    request: CommandChangePasswordRequest,
) -> CommandChangePasswordResponse {
    // Validate token and get user
    match state.auth_service.validate_token(&request.token) {
        Ok(user) => {
            use crate::application::ports::UserRepository;
            use crate::infrastructure::password;
            use crate::infrastructure::repositories::SqliteUserRepository;

            // Create a repository to update the user
            let pool = std::sync::Arc::clone(&state.auth_service.user_repository().pool());
            let user_repo = SqliteUserRepository::new(pool);

            // Verify current password
            if !password::verify_password(&request.current_password, &user.password_hash) {
                return CommandChangePasswordResponse {
                    success: false,
                    error: Some("Current password is incorrect".to_string()),
                };
            }

            // Validate new password is not empty
            if request.new_password.is_empty() {
                return CommandChangePasswordResponse {
                    success: false,
                    error: Some("New password cannot be empty".to_string()),
                };
            }

            // Hash new password
            let new_hash = match password::hash_password(&request.new_password) {
                Ok(h) => h,
                Err(e) => {
                    return CommandChangePasswordResponse {
                        success: false,
                        error: Some(format!("Failed to hash password: {}", e)),
                    };
                }
            };

            // Update user password
            match user_repo.find_by_id(&user.id) {
                Ok(Some(mut user_to_update)) => {
                    user_to_update.password_hash = new_hash;
                    user_to_update.updated_at = chrono::Utc::now();
                    if let Err(e) = user_repo.update(&user_to_update) {
                        return CommandChangePasswordResponse {
                            success: false,
                            error: Some(format!("Failed to update password: {}", e)),
                        };
                    }

                    CommandChangePasswordResponse {
                        success: true,
                        error: None,
                    }
                }
                Ok(None) => CommandChangePasswordResponse {
                    success: false,
                    error: Some("User not found".to_string()),
                },
                Err(e) => CommandChangePasswordResponse {
                    success: false,
                    error: Some(format!("Failed to find user: {}", e)),
                },
            }
        }
        Err(e) => CommandChangePasswordResponse {
            success: false,
            error: Some(e.to_string()),
        },
    }
}
