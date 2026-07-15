//! Settings SQLite Repository
//!
//! Implements SettingsRepository using SQLite.

use crate::application::ports::SettingsRepository;
use crate::domain::errors::DomainError;
use crate::infrastructure::database::SqlitePool;
use std::sync::Arc;

/// SQLite implementation of SettingsRepository
#[derive(Clone)]
pub struct SqliteSettingsRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteSettingsRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError> {
        let sql = "SELECT value FROM app_settings WHERE key = ?";

        match self
            .pool
            .query_row(sql, &[&key], |row| row.get::<_, String>(0))
        {
            Ok(value) => Ok(value),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Database(e.to_string())),
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let sql = "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, datetime('now'))";

        self.pool
            .execute(sql, &[&key, &value])
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }
}