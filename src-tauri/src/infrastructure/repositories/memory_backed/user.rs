//! MemoryBacked User Repository
//!
//! Implements UserRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5b: 6 MemoryBacked repositories.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::UserRepository;
use crate::domain::entities::user::{Role, User};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Email;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

/// MemoryBuffer-backed implementation of UserRepository.
#[derive(Clone)]
pub struct MemoryBackedUserRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedUserRepository {
    pub fn new(
        connection_manager: Arc<Mutex<ConnectionManager>>,
        memory_buffer: Arc<Mutex<MemoryBuffer>>,
        session: Arc<Mutex<CurrentSession>>,
    ) -> Self {
        Self {
            connection_manager,
            memory_buffer,
            session,
        }
    }

    /// Convert a libsql::Row into a HashMap<String, String> for cache storage.
    /// Column indices must match the SELECT statement order used in queries.
    fn row_to_hash_map(row: &libsql::Row) -> Result<HashMap<String, String>, DomainError> {
        let mut map = HashMap::new();
        map.insert(
            "id".to_string(),
            row.get::<String>(0)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "email".to_string(),
            row.get::<String>(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "password_hash".to_string(),
            row.get::<String>(2)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "name".to_string(),
            row.get::<String>(3)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "role".to_string(),
            row.get::<String>(4)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "is_active".to_string(),
            row.get::<i32>(5)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .to_string(),
        );
        map.insert(
            "created_at".to_string(),
            row.get::<String>(6)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "updated_at".to_string(),
            row.get::<String>(7)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        Ok(map)
    }

    fn row_to_user(row: &libsql::Row) -> Result<User, DomainError> {
        let role_str: String = row
            .get(4)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let is_active: i32 = row
            .get(5)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row
            .get(6)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row
            .get(7)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        let role = match role_str.as_str() {
            "Admin" => Role::Admin,
            "Gerente" => Role::Gerente,
            "Empleado" => Role::Empleado,
            "Profesor" => Role::Profesor,
            _ => Role::Admin,
        };

        Ok(User {
            id: row
                .get(0)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            email: row
                .get(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            password_hash: row
                .get(2)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            name: row
                .get(3)
                .map_err(|e| DomainError::Database(e.to_string()))?,
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

    fn user_from_data(data: &HashMap<String, String>) -> Result<User, DomainError> {
        let role_str = data
            .get("role")
            .ok_or_else(|| DomainError::Database("missing role".into()))?;
        let role = match role_str.as_str() {
            "Admin" => Role::Admin,
            "Gerente" => Role::Gerente,
            "Empleado" => Role::Empleado,
            "Profesor" => Role::Profesor,
            _ => Role::Admin,
        };

        let is_active: i32 = data
            .get("is_active")
            .ok_or_else(|| DomainError::Database("missing is_active".into()))?
            .parse()
            .unwrap_or(0);

        let created_str = data
            .get("created_at")
            .ok_or_else(|| DomainError::Database("missing created_at".into()))?;
        let updated_str = data
            .get("updated_at")
            .ok_or_else(|| DomainError::Database("missing updated_at".into()))?;

        Ok(User {
            id: data
                .get("id")
                .ok_or_else(|| DomainError::Database("missing id".into()))?
                .clone(),
            email: data
                .get("email")
                .ok_or_else(|| DomainError::Database("missing email".into()))?
                .clone(),
            password_hash: data
                .get("password_hash")
                .ok_or_else(|| DomainError::Database("missing password_hash".into()))?
                .clone(),
            name: data
                .get("name")
                .ok_or_else(|| DomainError::Database("missing name".into()))?
                .clone(),
            role,
            active: is_active != 0,
            created_at: DateTime::parse_from_rfc3339(created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn to_data_map(user: &User) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), user.id.clone());
        data.insert("email".to_string(), user.email.clone());
        data.insert("password_hash".to_string(), user.password_hash.clone());
        data.insert("name".to_string(), user.name.clone());
        data.insert(
            "role".to_string(),
            Self::role_to_string(user.role).to_string(),
        );
        data.insert(
            "is_active".to_string(),
            if user.active {
                "1".to_string()
            } else {
                "0".to_string()
            },
        );
        data.insert("created_at".to_string(), user.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), user.updated_at.to_rfc3339());
        data
    }

    async fn get_user_id(&self) -> Result<String, DomainError> {
        let session = self.session.lock().await;
        session
            .user_id
            .clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))
    }

    async fn query_turso(
        &self,
        user_id: &str,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<libsql::Rows, DomainError> {
        let db = {
            let cm = self.connection_manager.lock().await;
            cm.get_connection(user_id)
                .map(|c| c.db.clone())
                .ok_or_else(|| DomainError::Database("No connection for user".to_string()))?
        };
        let conn = db
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        conn.query(sql, params)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))
    }
}

#[async_trait]
impl UserRepository for MemoryBackedUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts/updates first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "users", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Ok(Some(Self::user_from_data(data)?));
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "users", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Ok(Some(Self::user_from_data(data)?));
                }
            }
            if buf.has_pending_delete(&user_id, "users", id) {
                return Ok(None);
            }
        }

        // Check entity cache or query Turso
        let row_data: Option<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_entity(&user_id, "users", id) {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql =
                    "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                           FROM users WHERE id = ?1";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![id]).await?;
                let data = match rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    Some(row) => Some(Self::row_to_hash_map(&row)?),
                    None => None,
                };

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_entity(&user_id, "users", id, data.clone());
                data
            }
        };

        match row_data {
            Some(data) => Ok(Some(Self::user_from_data(&data)?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let user_id = self.get_user_id().await?;
        let email_str = email.as_str().to_string();

        // Check pending inserts for matching email
        {
            let buf = self.memory_buffer.lock().await;
            let pending_inserts = buf.scan_pending_inserts(&user_id, "users");
            for op in pending_inserts.iter().rev() {
                if let BufferedOperation::Insert { data, .. } = op {
                    if data.get("email").map(|v| v.as_str()) == Some(&email_str) {
                        return Ok(Some(Self::user_from_data(data)?));
                    }
                }
            }
            // Check pending updates that change id
            let pending_updates = buf.scan_pending_updates(&user_id, "users");
            for op in pending_updates.iter().rev() {
                if let BufferedOperation::Update { data, .. } = op {
                    if data.get("email").map(|v| v.as_str()) == Some(&email_str) {
                        return Ok(Some(Self::user_from_data(data)?));
                    }
                }
            }
        }

        // Read from Turso
        let sql = "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                   FROM users WHERE email = ?1";
        let mut rows = self
            .query_turso(&user_id, sql, libsql::params![email.as_str()])
            .await?;
        match rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            Some(row) => Ok(Some(Self::row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;
        let data = Self::to_data_map(user);
        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "users".to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;
        let mut data = HashMap::new();
        data.insert("email".to_string(), user.email.clone());
        data.insert("password_hash".to_string(), user.password_hash.clone());
        data.insert("name".to_string(), user.name.clone());
        data.insert(
            "role".to_string(),
            Self::role_to_string(user.role).to_string(),
        );
        data.insert(
            "is_active".to_string(),
            if user.active {
                "1".to_string()
            } else {
                "0".to_string()
            },
        );
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        // Include id for deserialization
        data.insert("id".to_string(), user.id.clone());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "users".to_string(),
                id: user.id.clone(),
                data,
            },
        );
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        // Soft delete — update is_active to 0
        let mut data = HashMap::new();
        data.insert("is_active".to_string(), "0".to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("id".to_string(), id.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "users".to_string(),
                id: id.to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Step 1: Check cache or query Turso
        let base_rows: Vec<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_list(&user_id, "users") {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql =
                    "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at
                           FROM users WHERE is_active = 1 ORDER BY name";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

                let mut raw_rows: Vec<HashMap<String, String>> = Vec::new();
                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    raw_rows.push(Self::row_to_hash_map(&row)?);
                }

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_list(&user_id, "users", raw_rows.clone());
                raw_rows
            }
        };

        // Step 2: Convert rows to domain entities
        let mut results: Vec<User> = base_rows
            .iter()
            .map(|data| Self::user_from_data(data))
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "users");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                let user = Self::user_from_data(data)?;
                if user.active {
                    results.push(user);
                }
            }
        }

        let pending_updates = buf.scan_pending_updates(&user_id, "users");
        for op in &pending_updates {
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
                let updated_user = Self::user_from_data(data)?;
                if let Some(pos) = results.iter().position(|u| u.id == *update_id) {
                    if updated_user.active {
                        results[pos] = updated_user;
                    } else {
                        results.remove(pos);
                    }
                } else if updated_user.active {
                    results.push(updated_user);
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "users");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|u| u.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError> {
        let user_id = self.get_user_id().await?;
        let email_str = email.as_str().to_string();

        // Check pending inserts
        {
            let buf = self.memory_buffer.lock().await;
            let pending_inserts = buf.scan_pending_inserts(&user_id, "users");
            for op in pending_inserts.iter().rev() {
                if let BufferedOperation::Insert { data, .. } = op {
                    if data.get("email").map(|v| v.as_str()) == Some(&email_str) {
                        return Ok(true);
                    }
                }
            }
        }

        // Query Turso
        let sql = "SELECT COUNT(*) FROM users WHERE email = ?1";
        let mut rows = self
            .query_turso(&user_id, sql, libsql::params![email.as_str()])
            .await?;
        match rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            Some(row) => {
                let count: i32 = row
                    .get(0)
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(count > 0)
            }
            None => Ok(false),
        }
    }
}
