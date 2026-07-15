//! User SQLite Repository
//!
//! Implements UserRepository using SQLite.

use crate::application::ports::UserRepository;
use crate::domain::entities::user::{Role, User};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Email;
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of UserRepository
#[derive(Clone)]
pub struct SqliteUserRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteUserRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
        let role_str: String = row.get(4)?;
        let is_active: i32 = row.get(5)?;
        let created_str: String = row.get(6)?;
        let updated_str: String = row.get(7)?;

        let role = match role_str.as_str() {
            "Admin" => Role::Admin,
            "Gerente" => Role::Gerente,
            "Empleado" => Role::Empleado,
            "Profesor" => Role::Profesor,
            _ => Role::Admin,
        };

        Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            password_hash: row.get(2)?,
            name: row.get(3)?,
            role,
            active: is_active != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn role_to_string(role: Role) -> &'static str {
        match role {
            Role::Admin => "Admin",
            Role::Gerente => "Gerente",
            Role::Empleado => "Empleado",
            Role::Profesor => "Profesor",
        }
    }
}

impl UserRepository for SqliteUserRepository {
    fn pool(&self) -> Arc<SqlitePool> {
        Arc::clone(&self.pool)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE id = ?";

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        match conn.query_row(sql, [id], Self::row_to_user) {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE email = ?";

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        match conn.query_row(sql, [email.as_str()], Self::row_to_user) {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, user: &User) -> Result<(), DomainError> {
        let sql = "INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)";

        let role_str = Self::role_to_string(user.role);

        self.pool
            .execute(
                sql,
                &[
                    &user.id,
                    &user.email,
                    &user.password_hash,
                    &user.name,
                    &role_str,
                    &(if user.active { 1 } else { 0 }).to_string(),
                    &user.created_at.to_rfc3339(),
                    &user.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn update(&self, user: &User) -> Result<(), DomainError> {
        let sql = "UPDATE users 
                   SET email = ?, password_hash = ?, name = ?, role = ?, is_active = ?, updated_at = ?
                   WHERE id = ?";

        let role_str = Self::role_to_string(user.role);

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &user.email,
                    &user.password_hash,
                    &user.name,
                    &role_str,
                    &(if user.active { 1 } else { 0 }).to_string(),
                    &Utc::now().to_rfc3339(),
                    &user.id,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("User", &user.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE users SET is_active = 0, updated_at = ? WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&Utc::now().to_rfc3339(), &id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("User", id));
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<User>, DomainError> {
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE is_active = 1 ORDER BY name";

        self.pool
            .query(sql, &[], Self::row_to_user)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError> {
        let sql = "SELECT COUNT(*) FROM users WHERE email = ?";

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let count: i32 = conn
            .query_row(sql, [email.as_str()], |row| row.get(0))
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(count > 0)
    }
}
