//! MemoryBacked Settings Repository
//!
//! Implements SettingsRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5a: Foundation — reads from Turso, writes to MemoryBuffer.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::SettingsRepository;
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;

/// MemoryBuffer-backed implementation of SettingsRepository.
///
/// - `get_setting`: checks the pending buffer first (for unflushed writes),
///   then falls through to the user's Turso database.
/// - `set_setting`: writes to the MemoryBuffer (lazy-flush to Turso).
pub struct MemoryBackedSettingsRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedSettingsRepository {
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
}

#[async_trait]
impl SettingsRepository for MemoryBackedSettingsRepository {
    async fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError> {
        let user_id = {
            let session = self.session.lock().await;
            session
                .user_id
                .clone()
                .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?
        };

        // Check pending buffer first — newest writes shadow older ones
        {
            let buf = self.memory_buffer.lock().await;
            let pending = buf.scan_pending_inserts(&user_id, "app_settings");
            // Iterate in reverse (most recent first) to find the latest value
            for op in pending.iter().rev() {
                if let BufferedOperation::Insert { data, .. } = op {
                    if data.get("key").map(|v| v.as_str()) == Some(key) {
                        return Ok(data.get("value").cloned());
                    }
                }
            }
        }

        // Read from Turso DB
        let db = {
            let cm = self.connection_manager.lock().await;
            cm.get_connection(&user_id)
                .map(|c| c.db.clone())
                .ok_or_else(|| DomainError::Database("No connection for user".to_string()))?
        };

        let conn = db
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn
            .query(
                "SELECT value FROM app_settings WHERE key = ?1",
                libsql::params![key.to_owned()],
            )
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        match rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            Some(row) => {
                let value: String = row
                    .get(0)
                    .map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let user_id = {
            let session = self.session.lock().await;
            session
                .user_id
                .clone()
                .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?
        };

        let mut data = HashMap::new();
        data.insert("key".to_string(), key.to_string());
        data.insert("value".to_string(), value.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "app_settings".to_string(),
                data,
            },
        );

        Ok(())
    }
}
