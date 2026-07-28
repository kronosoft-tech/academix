use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{CreateUserRequest, UpdateUserRequest, UserDto, UserListItem};
use crate::application::use_cases::UserService;
use crate::infrastructure::repositories::SqliteUserRepository;

pub type UserServiceState = UserService<SqliteUserRepository>;

#[derive(Debug, Deserialize)]
pub struct CreateUserCommand {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserCommand {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserCommandResponse {
    pub success: bool,
    pub data: Option<UserDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<UserListItem>>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn create_user(
    state: State<'_, UserServiceState>,
    request: CreateUserCommand,
) -> Result<UserCommandResponse, String> {
    match state.create(CreateUserRequest {
        email: request.email,
        password: request.password,
        name: request.name,
        role: request.role,
    }).await {
        Ok(user) => Ok(UserCommandResponse {
            success: true,
            data: Some(user),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_user(state: State<'_, UserServiceState>, id: String) -> Result<UserCommandResponse, String> {
    match state.get_by_id(&id).await {
        Ok(user) => Ok(UserCommandResponse {
            success: true,
            data: Some(user),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_users(state: State<'_, UserServiceState>) -> Result<UserListCommandResponse, String> {
    match state.list().await {
        Ok(users) => Ok(UserListCommandResponse {
            success: true,
            data: Some(users),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_users_by_role(state: State<'_, UserServiceState>, role: String) -> Result<UserListCommandResponse, String> {
    match state.list_by_role(&role).await {
        Ok(users) => Ok(UserListCommandResponse {
            success: true,
            data: Some(users),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_user(
    state: State<'_, UserServiceState>,
    id: String,
    request: UpdateUserCommand,
) -> Result<UserCommandResponse, String> {
    if request.role.is_some() || request.password.is_some() {
        match state.admin_update(
            &id,
            UpdateUserRequest {
                name: request.name,
                email: request.email,
                role: request.role,
                password: request.password,
            },
        ).await {
            Ok(user) => Ok(UserCommandResponse {
                success: true,
                data: Some(user),
                error: None,
            }),
            Err(e) => Err(e.to_string()),
        }
    } else {
        match state.update(
            &id,
            UpdateUserRequest {
                name: request.name,
                email: request.email,
                role: None,
                password: None,
            },
        ).await {
            Ok(user) => Ok(UserCommandResponse {
                success: true,
                data: Some(user),
                error: None,
            }),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[tauri::command]
pub async fn delete_user(state: State<'_, UserServiceState>, id: String) -> Result<UserCommandResponse, String> {
    match state.delete(&id).await {
        Ok(()) => Ok(UserCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}
