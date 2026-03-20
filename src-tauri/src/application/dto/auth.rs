//! Authentication DTOs

use serde::{Deserialize, Serialize};

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserDto,
    pub expires_at: String,
}

/// Logout request
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub token: String,
}

/// User DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}
