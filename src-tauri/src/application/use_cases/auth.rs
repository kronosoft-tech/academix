//! Authentication Use Cases
//!
//! Handles user login, logout, and session management.

use crate::application::dto::{LoginRequest, LoginResponse, UserDto};
use crate::application::errors::ApplicationError;
use crate::application::ports::{SessionRepository, UserRepository};
use crate::domain::entities::{Session, User};
use crate::domain::value_objects::Email;
use crate::infrastructure::password;
use chrono::{Duration, Utc};
use uuid::Uuid;

/// Authentication service
pub struct AuthService<U: UserRepository, S: SessionRepository> {
    user_repository: U,
    session_repository: S,
}

impl<U: UserRepository, S: SessionRepository> AuthService<U, S> {
    pub fn new(user_repository: U, session_repository: S) -> Self {
        Self {
            user_repository,
            session_repository,
        }
    }

    /// Authenticate user and create session
    pub fn login(&self, request: LoginRequest) -> Result<LoginResponse, ApplicationError> {
        // Find user by email
        let email = Email::new(&request.email).map_err(|e| ApplicationError::Validation(e))?;

        let user = self
            .user_repository
            .find_by_email(&email)?
            .ok_or_else(|| ApplicationError::Authentication("Invalid credentials".to_string()))?;

        // Verify password
        if !password::verify_password(&request.password, &user.password_hash) {
            return Err(ApplicationError::Authentication(
                "Invalid credentials".to_string(),
            ));
        }

        // Check if user is active
        if !user.active {
            return Err(ApplicationError::Authentication(
                "User account is inactive".to_string(),
            ));
        }

        // Create session
        let session_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();
        let token = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();
        let expires_at = Utc::now() + Duration::hours(24);

        let session = Session::new(session_id, user.id.clone(), token.clone(), expires_at);
        self.session_repository.save(&session)?;

        Ok(LoginResponse {
            token,
            user: UserDto {
                id: user.id,
                email: user.email,
                name: user.name,
                role: user.role.as_str().to_string(),
            },
            expires_at: expires_at.to_rfc3339(),
        })
    }

    /// Logout user and destroy session
    pub fn logout(&self, token: &str) -> Result<(), ApplicationError> {
        let session = self
            .session_repository
            .find_by_token(token)?
            .ok_or_else(|| ApplicationError::Authentication("Session not found".to_string()))?;

        self.session_repository.delete(&session.id)?;

        Ok(())
    }

    /// Validate session token
    pub fn validate_token(&self, token: &str) -> Result<User, ApplicationError> {
        let session = self
            .session_repository
            .find_by_token(token)?
            .ok_or_else(|| ApplicationError::Authentication("Invalid token".to_string()))?;

        if !session.is_valid() {
            return Err(ApplicationError::Authentication(
                "Token expired".to_string(),
            ));
        }

        let user = self
            .user_repository
            .find_by_id(&session.user_id)?
            .ok_or_else(|| ApplicationError::Authentication("User not found".to_string()))?;

        if !user.active {
            return Err(ApplicationError::Authentication(
                "User account is inactive".to_string(),
            ));
        }

        Ok(user)
    }
}
