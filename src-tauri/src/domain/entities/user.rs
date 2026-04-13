//! User Entity - Domain Model
//!
//! Pure domain entity with no persistence concerns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User roles in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Gerente,
    Empleado,
    Profesor,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Gerente => "gerente",
            Role::Empleado => "empleado",
            Role::Profesor => "profesor",
        }
    }

    pub fn from_str(s: &str) -> Option<Role> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "gerente" => Some(Role::Gerente),
            "empleado" => Some(Role::Empleado),
            "profesor" => Some(Role::Profesor),
            _ => None,
        }
    }
}

/// User entity - core domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: Role,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user (password should be hashed before saving)
    pub fn new(id: String, email: String, password_hash: String, name: String, role: Role) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            password_hash,
            name,
            role,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: Role) -> bool {
        self.role == role
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    /// Deactivate user (soft delete)
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    /// Update user profile
    pub fn update_profile(&mut self, name: String, email: String) {
        self.name = name;
        self.email = email;
        self.updated_at = Utc::now();
    }
}

/// Session entity for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(id: String, user_id: String, token: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id,
            user_id,
            token,
            expires_at,
            created_at: Utc::now(),
        }
    }

    /// Check if session is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test-id".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
            Role::Admin,
        );

        assert_eq!(user.id, "test-id");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.role, Role::Admin);
        assert!(user.active);
    }

    #[test]
    fn test_has_role() {
        let user = User::new(
            "test-id".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
            Role::Empleado,
        );

        assert!(user.has_role(Role::Empleado));
        assert!(!user.has_role(Role::Admin));
    }

    #[test]
    fn test_is_admin() {
        let admin = User::new(
            "admin-id".to_string(),
            "admin@example.com".to_string(),
            "hash".to_string(),
            "Admin".to_string(),
            Role::Admin,
        );

        let empleado = User::new(
            "emp-id".to_string(),
            "emp@example.com".to_string(),
            "hash".to_string(),
            "Empleado".to_string(),
            Role::Empleado,
        );

        assert!(admin.is_admin());
        assert!(!empleado.is_admin());
    }

    #[test]
    fn test_deactivate_user() {
        let mut user = User::new(
            "test-id".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            "Test User".to_string(),
            Role::Admin,
        );

        assert!(user.active);

        user.deactivate();

        assert!(!user.active);
    }

    #[test]
    fn test_update_profile() {
        let mut user = User::new(
            "test-id".to_string(),
            "old@example.com".to_string(),
            "hashed_password".to_string(),
            "Old Name".to_string(),
            Role::Admin,
        );

        user.update_profile("New Name".to_string(), "new@example.com".to_string());

        assert_eq!(user.name, "New Name");
        assert_eq!(user.email, "new@example.com");
    }

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::Admin.as_str(), "admin");
        assert_eq!(Role::Gerente.as_str(), "gerente");
        assert_eq!(Role::Empleado.as_str(), "empleado");
        assert_eq!(Role::Profesor.as_str(), "profesor");
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("ADMIN"), Some(Role::Admin));
        assert_eq!(Role::from_str("gerente"), Some(Role::Gerente));
        assert_eq!(Role::from_str("unknown"), None);
    }
}
