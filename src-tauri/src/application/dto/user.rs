//! User DTOs

use serde::{Deserialize, Serialize};

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: String,
}

/// Register user request (public registration)
#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

/// Register user response
#[derive(Debug, Serialize)]
pub struct RegisterUserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

/// Update user request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

/// Admin update user request (includes role and optional password reset)
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

/// User list item
#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub active: bool,
}
