//! Registration Use Case
//!
//! Handles user registration with validation and password hashing.

use crate::application::dto::{RegisterUserRequest, RegisterUserResponse};
use crate::application::errors::ApplicationError;
use crate::application::ports::UserRepository;
use crate::domain::entities::{Role, User};
use crate::domain::value_objects::{Email, Password};
use crate::infrastructure::password;
use uuid::Uuid;

/// Registration use case
pub struct RegisterUserUseCase<R: UserRepository> {
    user_repository: R,
}

impl<R: UserRepository> RegisterUserUseCase<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }

    /// Register a new user with default role (not admin)
    pub fn execute(&self, request: RegisterUserRequest) -> Result<RegisterUserResponse, ApplicationError> {
        // Validate email format
        let email = Email::new(&request.email).map_err(|e| ApplicationError::Validation(e))?;

        // Validate password minimum 8 characters
        Password::validate_strength(&request.password)
            .map_err(|e| ApplicationError::Validation(e))?;

        // Check if email already exists
        if self.user_repository.exists_by_email(&email)? {
            return Err(ApplicationError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        // Hash password
        let password_hash = password::hash_password(&request.password)
            .map_err(|e| ApplicationError::Infrastructure(e.to_string()))?;

        // Create user with default role (empleado - not admin)
        let user = User::new(
            Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            request.email,
            password_hash,
            request.name,
            Role::Empleado, // Default role - not admin for self-registration
        );

        // Save to repository
        self.user_repository.save(&user)?;

        Ok(RegisterUserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
        })
    }
}