//! MemoryBacked Session Repository
//!
//! Implements SessionRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5b: 6 MemoryBacked repositories.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::SessionRepository;
use crate::domain::entities::Session;
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

/// MemoryBuffer-backed implementation of SessionRepository.
pub struct MemoryBackedSessionRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedSessionRepository {
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

    fn row_to_session(row: &libsql::Row) -> Result<Session, DomainError> {
        let expires_str: String = row.get(3).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(4).map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(Session {
            id: row.get(0).map_err(|e| DomainError::Database(e.to_string()))?,
            user_id: row.get(1).map_err(|e| DomainError::Database(e.to_string()))?,
            token: row.get(2).map_err(|e| DomainError::Database(e.to_string()))?,
            expires_at: DateTime::parse_from_rfc3339(&expires_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn session_from_data(data: &HashMap<String, String>) -> Result<Session, DomainError> {
        let expires_str = data.get("expires_at").ok_or_else(|| DomainError::Database("missing expires_at".into()))?;
        let created_str = data.get("created_at").ok_or_else(|| DomainError::Database("missing created_at".into()))?;

        Ok(Session {
            id: data.get("id").ok_or_else(|| DomainError::Database("missing id".into()))?.clone(),
            user_id: data.get("user_id").ok_or_else(|| DomainError::Database("missing user_id".into()))?.clone(),
            token: data.get("token").ok_or_else(|| DomainError::Database("missing token".into()))?.clone(),
            expires_at: DateTime::parse_from_rfc3339(expires_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created_at: DateTime::parse_from_rfc3339(created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    async fn get_user_id(&self) -> Result<String, DomainError> {
        let session = self.session.lock().await;
        session
            .user_id
            .clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))
    }

    async fn query_turso(&self, user_id: &str, sql: &str, params: impl libsql::params::IntoParams) -> Result<libsql::Rows, DomainError> {
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
impl SessionRepository for MemoryBackedSessionRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts/updates first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "sessions", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Ok(Some(Self::session_from_data(data)?));
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "sessions", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Ok(Some(Self::session_from_data(data)?));
                }
            }
            if buf.has_pending_delete(&user_id, "sessions", id) {
                return Ok(None);
            }
        }

        // Read from Turso
        let sql = "SELECT id, user_id, token, expires_at, created_at
                   FROM sessions WHERE id = ?1";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![id]).await?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_session(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts via the token-specific method first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some((_uid, data)) = buf.find_session_by_token(token) {
                return Ok(Some(Self::session_from_data(&data)?));
            }
        }

        // Read from Turso
        let sql = "SELECT id, user_id, token, expires_at, created_at
                   FROM sessions WHERE token = ?1";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![token]).await?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_session(&row)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, session: &Session) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut data = HashMap::new();
        data.insert("id".to_string(), session.id.clone());
        data.insert("user_id".to_string(), session.user_id.clone());
        data.insert("token".to_string(), session.token.clone());
        data.insert("expires_at".to_string(), session.expires_at.to_rfc3339());
        data.insert("created_at".to_string(), session.created_at.to_rfc3339());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Insert {
            table: "sessions".to_string(),
            data,
        });
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Delete {
            table: "sessions".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, DomainError> {
        let user_id = self.get_user_id().await?;
        let now = Utc::now().to_rfc3339();

        // Execute against Turso directly (this is a bulk cleanup operation)
        let sql = "DELETE FROM sessions WHERE expires_at < ?1";
        let db = {
            let cm = self.connection_manager.lock().await;
            cm.get_connection(&user_id)
                .map(|c| c.db.clone())
                .ok_or_else(|| DomainError::Database("No connection for user".to_string()))?
        };
        let conn = db
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![now])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(affected as u64)
    }
}
