//! Settings SQLite Repository
//!
//! Implements SettingsRepository using SQLite.

use async_trait::async_trait;
use crate::application::ports::SettingsRepository;
use crate::domain::errors::DomainError;
use crate::infrastructure::local_db;

/// SQLite implementation of SettingsRepository
#[derive(Clone)]
pub struct SqliteSettingsRepository;

impl SqliteSettingsRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteSettingsRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepository {
    async fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError> {
        let sql = "SELECT value FROM app_settings WHERE key = ?";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![key.to_owned()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => {
                let value: String = row.get(0).map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let sql = "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, datetime('now'))";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(sql, libsql::params![key.to_owned(), value.to_owned()])
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }
}
