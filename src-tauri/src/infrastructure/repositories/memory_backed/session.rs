//! MemoryBuffer-backed Session Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::SessionRepository;
use crate::domain::entities::Session;
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedSessionRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedSessionRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(session: &Session) -> CachedEntity {
        CachedEntity {
            id: session.id.clone(),
            data: HashMap::from([
                ("id".to_string(), session.id.clone()),
                ("user_id".to_string(), session.user_id.clone()),
                ("token".to_string(), session.token.clone()),
                ("expires_at".to_string(), session.expires_at.to_rfc3339()),
                ("created_at".to_string(), session.created_at.to_rfc3339()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<Session> {
        Some(Session {
            id: cached.data.get("id")?.clone(),
            user_id: cached.data.get("user_id")?.clone(),
            token: cached.data.get("token")?.clone(),
            expires_at: cached.data.get("expires_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
        })
    }

    fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
        let expires_str: String = row.get(3)?;
        let created_str: String = row.get(4)?;

        Ok(Session {
            id: row.get(0)?,
            user_id: row.get(1)?,
            token: row.get(2)?,
            expires_at: DateTime::parse_from_rfc3339(&expires_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl SessionRepository for MemoryBackedSessionRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError> {
        let cache_key = format!("session:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, user_id, token, expires_at, created_at 
                 FROM sessions WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_session) {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_token(&self, token: &str) -> Result<Option<Session>, DomainError> {
        let cache_key = format!("session:token:{}", token);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, user_id, token, expires_at, created_at 
                 FROM sessions WHERE token = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [token], Self::row_to_session) {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, session: &Session) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(session).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "sessions".to_string(),
            data,
        });
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "sessions".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    fn delete_expired(&self) -> Result<u64, DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        // Buffer a delete operation — the actual deletion of expired sessions will
        // be handled on flush by filtering against current time.
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "sessions".to_string(),
            id: "__expired__".to_string(),
        });
        Ok(0)
    }
}
