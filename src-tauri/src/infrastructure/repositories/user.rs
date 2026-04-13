//! In-Memory User Repository

use std::collections::HashMap;
use std::sync::RwLock;

use crate::application::ports::UserRepository;
use crate::domain::entities::User;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Email;

/// In-memory user repository implementation
pub struct InMemoryUserRepository {
    users: RwLock<HashMap<String, User>>,
    emails: RwLock<HashMap<String, String>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        let mut repo = Self {
            users: RwLock::new(HashMap::new()),
            emails: RwLock::new(HashMap::new()),
        };
        // Seed admin user (password: 123456!)
        repo.seed_admin();
        repo
    }

    /// Seed the admin user for initial access
    fn seed_admin(&mut self) {
        let admin = User::new(
            "1".to_string(),
            "admin@academix.com".to_string(),
            // bcrypt hash for "admin123"
            "$2b$12$gghetCr2w7EqfgK5u8jMru4Malw8kQZcXMUQfp2dwOsac2xlo5gYy".to_string(),
            "Luifer Admin".to_string(),
            crate::domain::entities::user::Role::Admin,
        );

        let mut users = self.users.write().unwrap();
        let mut emails = self.emails.write().unwrap();

        users.insert(admin.id.clone(), admin);
        emails.insert("admin@academix.com".to_string(), "1".to_string());
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl UserRepository for InMemoryUserRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        let users = self
            .users
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(users.get(id).cloned())
    }

    fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let emails = self
            .emails
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let users = self
            .users
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(id) = emails.get(email.as_str()) {
            Ok(users.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn save(&self, user: &User) -> Result<(), DomainError> {
        let mut users = self
            .users
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut emails = self
            .emails
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        users.insert(user.id.clone(), user.clone());
        emails.insert(user.email.clone(), user.id.clone());

        Ok(())
    }

    fn update(&self, user: &User) -> Result<(), DomainError> {
        let mut users = self
            .users
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if !users.contains_key(&user.id) {
            return Err(DomainError::not_found("User", &user.id));
        }

        users.insert(user.id.clone(), user.clone());

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut users = self
            .users
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(user) = users.get(id) {
            let mut emails = self
                .emails
                .write()
                .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
            emails.remove(&user.email);
            users.remove(id);
        }

        Ok(())
    }

    fn find_all(&self) -> Result<Vec<User>, DomainError> {
        let users = self
            .users
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(users.values().cloned().collect())
    }

    fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError> {
        let emails = self
            .emails
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(emails.contains_key(email.as_str()))
    }
}
