//! MemoryBuffer-backed Settings Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.
//! Settings use a simple key-value pattern.

use crate::application::ports::SettingsRepository;
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedSettingsRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedSettingsRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn setting_cache_key(key: &str) -> String {
        format!("app_setting:{}", key)
    }

    fn to_cached(key: &str, value: &str) -> CachedEntity {
        CachedEntity {
            id: key.to_string(),
            data: HashMap::from([
                ("key".to_string(), key.to_string()),
                ("value".to_string(), value.to_string()),
            ]),
        }
    }

    fn value_from_cached(cached: &CachedEntity) -> Option<String> {
        cached.data.get("value").cloned()
    }
}

impl SettingsRepository for MemoryBackedSettingsRepository {
    fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError> {
        let cache_key = Self::setting_cache_key(key);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(value) = Self::value_from_cached(cached) {
                    return Ok(Some(value));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT value FROM app_settings WHERE key = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [key], |row| row.get::<_, String>(0)) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Database(e.to_string())),
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(key, value).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "app_settings".to_string(),
            data,
        });
        Ok(())
    }
}
