//! User SQLite Repository
//!
//! Implements UserRepository using libSQL (async).

use async_trait::async_trait;
use libsql::params::IntoParams;
use crate::application::ports::UserRepository;
use crate::domain::entities::user::{Role, User};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Email;
use crate::infrastructure::local_db;
use chrono::{DateTime, Utc};

/// SQLite implementation of UserRepository
#[derive(Clone)]
pub struct SqliteUserRepository;

impl SqliteUserRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_user(row: &libsql::Row) -> Result<User, DomainError> {
        let role_str: String = row.get(4).map_err(|e| DomainError::Database(e.to_string()))?;
        let is_active: i32 = row.get(5).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(6).map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row.get(7).map_err(|e| DomainError::Database(e.to_string()))?;

        let role = match role_str.as_str() {
            "Admin" => Role::Admin,
            "Gerente" => Role::Gerente,
            "Empleado" => Role::Empleado,
            "Profesor" => Role::Profesor,
            _ => Role::Admin,
        };

        Ok(User {
            id: row.get(0).map_err(|e| DomainError::Database(e.to_string()))?,
            email: row.get(1).map_err(|e| DomainError::Database(e.to_string()))?,
            password_hash: row.get(2).map_err(|e| DomainError::Database(e.to_string()))?,
            name: row.get(3).map_err(|e| DomainError::Database(e.to_string()))?,
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

    async fn query_one<F, T>(sql: &str, params: impl IntoParams, mapper: F) -> Result<Option<T>, DomainError>
    where
        F: Fn(&libsql::Row) -> Result<T, DomainError>,
    {
        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, params).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(mapper(&row)?)),
            None => Ok(None),
        }
    }
}

impl Default for SqliteUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE id = ?1";
        Self::query_one(sql, libsql::params![id], Self::row_to_user).await
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE email = ?1";
        Self::query_one(sql, libsql::params![email.as_str()], Self::row_to_user).await
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let sql = "INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)";

        let role_str = Self::role_to_string(user.role);
        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(
            sql,
            libsql::params![
                user.id.clone(),
                user.email.clone(),
                user.password_hash.clone(),
                user.name.clone(),
                role_str,
                if user.active { 1 } else { 0 },
                user.created_at.to_rfc3339(),
                user.updated_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        let sql = "UPDATE users 
                   SET email = ?1, password_hash = ?2, name = ?3, role = ?4, is_active = ?5, updated_at = ?6
                   WHERE id = ?7";

        let role_str = Self::role_to_string(user.role);
        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    user.email.clone(),
                    user.password_hash.clone(),
                    user.name.clone(),
                    role_str,
                    if user.active { 1 } else { 0 },
                    Utc::now().to_rfc3339(),
                    user.id.clone(),
                ],
            )
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("User", &user.id));
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE users SET is_active = 0, updated_at = ?1 WHERE id = ?2";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![Utc::now().to_rfc3339(), id])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("User", id));
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, DomainError> {
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE is_active = 1 ORDER BY name";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, ()).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_user(&row)?);
        }
        Ok(results)
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError> {
        let sql = "SELECT COUNT(*) FROM users WHERE email = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![email.as_str()]).await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => {
                let count: i32 = row.get(0).map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(count > 0)
            }
            None => Ok(false),
        }
    }
}
