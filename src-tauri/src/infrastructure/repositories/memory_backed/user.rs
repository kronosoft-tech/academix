//! MemoryBuffer-backed User Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::UserRepository;
use crate::domain::entities::user::{Role, User};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Email;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedUserRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedUserRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(user: &User) -> CachedEntity {
        CachedEntity {
            id: user.id.clone(),
            data: HashMap::from([
                ("id".to_string(), user.id.clone()),
                ("email".to_string(), user.email.clone()),
                ("password_hash".to_string(), user.password_hash.clone()),
                ("name".to_string(), user.name.clone()),
                ("role".to_string(), user.role.as_str().to_string()),
                ("active".to_string(), if user.active { "1" } else { "0" }.to_string()),
                ("created_at".to_string(), user.created_at.to_rfc3339()),
                ("updated_at".to_string(), user.updated_at.to_rfc3339()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<User> {
        let role_str = cached.data.get("role")?;
        let role = Role::from_str(role_str).unwrap_or(Role::Admin);
        Some(User {
            id: cached.data.get("id")?.clone(),
            email: cached.data.get("email")?.clone(),
            password_hash: cached.data.get("password_hash")?.clone(),
            name: cached.data.get("name")?.clone(),
            role,
            active: cached.data.get("active")? == "1",
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            updated_at: cached.data.get("updated_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
        })
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
}

impl UserRepository for MemoryBackedUserRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        let cache_key = format!("user:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_user) {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let cache_key = format!("user:email:{}", email.as_str());
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE email = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [email.as_str()], Self::row_to_user) {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, user: &User) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(user).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "users".to_string(),
            data,
        });
        Ok(())
    }

    fn update(&self, user: &User) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(user).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "users".to_string(),
            id: user.id.clone(),
            data,
        });
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "users".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<User>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE is_active = 1 ORDER BY name";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_user)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<User>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError> {
        // No caching for count queries - go directly to SQLite
        let sql = "SELECT COUNT(*) FROM users WHERE email = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let count: i32 = conn
            .query_row(sql, [email.as_str()], |row| row.get(0))
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(count > 0)
    }
}
