//! Settings SQLite Repository
//!
//! Implements SettingsRepository using SQLite.

use crate::application::ports::SettingsRepository;
use crate::domain::errors::DomainError;
use crate::infrastructure::database;

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

impl SettingsRepository for SqliteSettingsRepository {
    fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError> {
        let sql = "SELECT value FROM app_settings WHERE key = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [key], |row| row.get::<_, String>(0)) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Database(e.to_string())),
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError> {
        let sql = "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, datetime('now'))";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        conn.execute(sql, rusqlite::params![key, value])
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }
}
