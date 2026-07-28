//! SQLite Session Repository

use crate::application::ports::SessionRepository;
use crate::domain::entities::Session;
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use chrono::{DateTime, Utc};

/// SQLite implementation of SessionRepository
#[derive(Clone)]
pub struct SqliteSessionRepository;

impl SqliteSessionRepository {
    pub fn new() -> Self {
        Self
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

impl Default for SqliteSessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError> {
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
        let sql = "INSERT INTO sessions (id, user_id, token, expires_at, created_at)
                 VALUES (?, ?, ?, ?, ?)";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        conn.execute(
            sql,
            rusqlite::params![
                session.id,
                session.user_id,
                session.token,
                session.expires_at.to_rfc3339(),
                session.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM sessions WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let affected = conn
            .execute(sql, rusqlite::params![id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Session", id));
        }
        Ok(())
    }

    fn delete_expired(&self) -> Result<u64, DomainError> {
        let now = Utc::now().to_rfc3339();
        let sql = "DELETE FROM sessions WHERE expires_at < ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let affected = conn
            .execute(sql, rusqlite::params![now])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(affected as u64)
    }
}
