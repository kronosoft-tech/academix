//! User Commands
//!
//! Tauri commands for user management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CreateUserRequest, UpdateUserRequest, UserDto, UserListItem};
use crate::application::use_cases::UserService;
use crate::infrastructure::repositories::SqliteUserRepository;

pub type UserServiceState = UserService<SqliteUserRepository>;

/// Create user request payload
#[derive(Debug, Deserialize)]
pub struct CreateUserCommand {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: String,
}

/// Update user request payload
#[derive(Debug, Deserialize)]
pub struct UpdateUserCommand {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// User response payload
#[derive(Debug, Serialize)]
pub struct UserCommandResponse {
    pub success: bool,
    pub data: Option<UserDto>,
    pub error: Option<String>,
}

/// User list response
#[derive(Debug, Serialize)]
pub struct UserListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<UserListItem>>,
    pub error: Option<String>,
}

/// Create user command
#[tauri::command]
pub fn create_user(
    state: State<UserServiceState>,
    request: CreateUserCommand,
) -> UserCommandResponse {
    match state.create(CreateUserRequest {
        email: request.email,
        password: request.password,
        name: request.name,
        role: request.role,
    }) {
        Ok(user) => UserCommandResponse {
            success: true,
            data: Some(user),
            error: None,
        },
        Err(e) => UserCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Get user by ID
#[tauri::command]
pub fn get_user(state: State<UserServiceState>, id: String) -> UserCommandResponse {
    match state.get_by_id(&id) {
        Ok(user) => UserCommandResponse {
            success: true,
            data: Some(user),
            error: None,
        },
        Err(e) => UserCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List all users
#[tauri::command]
pub fn list_users(state: State<UserServiceState>) -> UserListCommandResponse {
    match state.list() {
        Ok(users) => UserListCommandResponse {
            success: true,
            data: Some(users),
            error: None,
        },
        Err(e) => UserListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Update user
#[tauri::command]
pub fn update_user(
    state: State<UserServiceState>,
    id: String,
    request: UpdateUserCommand,
) -> UserCommandResponse {
    match state.update(
        &id,
        UpdateUserRequest {
            name: request.name,
            email: request.email,
        },
    ) {
        Ok(user) => UserCommandResponse {
            success: true,
            data: Some(user),
            error: None,
        },
        Err(e) => UserCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Delete user
#[tauri::command]
pub fn delete_user(state: State<UserServiceState>, id: String) -> UserCommandResponse {
    match state.delete(&id) {
        Ok(()) => UserCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => UserCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}
