//! User Use Cases

use crate::application::dto::{CreateUserRequest, UpdateUserRequest, UserDto, UserListItem};
use crate::application::errors::ApplicationError;
use crate::application::ports::UserRepository;
use crate::domain::entities::{Role, User};
use crate::domain::value_objects::Email;
use crate::infrastructure::password;
use uuid::Uuid;

/// User service
pub struct UserService<R: UserRepository> {
    user_repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }

    /// Create a new user
    pub fn create(&self, request: CreateUserRequest) -> Result<UserDto, ApplicationError> {
        // Validate email
        let email = Email::new(&request.email).map_err(|e| ApplicationError::Validation(e))?;

        // Check if email already exists
        if self.user_repository.exists_by_email(&email)? {
            return Err(ApplicationError::Conflict(
                "Email already exists".to_string(),
            ));
        }

        // Validate role
        let role = Role::from_str(&request.role)
            .ok_or_else(|| ApplicationError::Validation("Invalid role".to_string()))?;

        // Hash password
        let password_hash = password::hash_password(&request.password)
            .map_err(|e| ApplicationError::Infrastructure(e.to_string()))?;

        // Create user entity
        let user = User::new(
            Uuid::new_v4().to_string(),
            request.email,
            password_hash,
            request.name,
            role,
        );

        // Save to repository
        self.user_repository.save(&user)?;

        Ok(UserDto {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
        })
    }

    /// Get user by ID
    pub fn get_by_id(&self, id: &str) -> Result<UserDto, ApplicationError> {
        let user = self
            .user_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("User not found".to_string()))?;

        Ok(UserDto {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
        })
    }

    /// List all users
    pub fn list(&self) -> Result<Vec<UserListItem>, ApplicationError> {
        let users = self.user_repository.find_all()?;

        Ok(users
            .into_iter()
            .map(|u| UserListItem {
                id: u.id,
                email: u.email,
                name: u.name,
                role: u.role.as_str().to_string(),
                active: u.active,
            })
            .collect())
    }

    /// Update user
    pub fn update(
        &self,
        id: &str,
        request: UpdateUserRequest,
    ) -> Result<UserDto, ApplicationError> {
        let mut user = self
            .user_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("User not found".to_string()))?;

        if let Some(name) = request.name {
            user.name = name;
        }

        if let Some(email) = request.email {
            let email = Email::new(&email).map_err(|e| ApplicationError::Validation(e))?;

            // Check if email changed and if new email already exists
            if email.as_str() != user.email {
                if self.user_repository.exists_by_email(&email)? {
                    return Err(ApplicationError::Conflict(
                        "Email already exists".to_string(),
                    ));
                }
                user.email = email.to_string();
            }
        }

        self.user_repository.update(&user)?;

        Ok(UserDto {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
        })
    }

    /// Delete user (soft delete)
    pub fn delete(&self, id: &str) -> Result<(), ApplicationError> {
        self.user_repository.delete(id)?;
        Ok(())
    }
}
