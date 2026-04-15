//! SQLite Session Repository

use crate::application::ports::SessionRepository;
use crate::domain::entities::Session;
use crate::domain::errors::DomainError;
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of SessionRepository
#[derive(Clone)]
pub struct SqliteSessionRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteSessionRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
        let expires_str: String = row.get(4)?;
        let created_str: String = row.get(5)?;

        Ok(Session {
            id: row.get(0)?,
            user_id: row.get(1)?,
            token: row.get(2)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            expires_at: DateTime::parse_from_rfc3339(&expires_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError> {
        let sql = "SELECT id, user_id, token, expires_at, created_at 
                 FROM sessions WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_session)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_token(&self, token: &str) -> Result<Option<Session>, DomainError> {
        let sql = "SELECT id, user_id, token, expires_at, created_at 
                 FROM sessions WHERE token = ?";

        self.pool
            .query_row(sql, &[&token], Self::row_to_session)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn save(&self, session: &Session) -> Result<(), DomainError> {
        let sql = "INSERT INTO sessions (id, user_id, token, expires_at, created_at)
                 VALUES (?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &session.id,
                    &session.user_id,
                    &session.token,
                    &session.expires_at.to_rfc3339(),
                    &session.created_at.to_rfc3339(),
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM sessions WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Session", id));
        }
        Ok(())
    }

    fn delete_expired(&self) -> Result<u64, DomainError> {
        let now = Utc::now().to_rfc3339();
        let sql = "DELETE FROM sessions WHERE expires_at < ?";

        let affected = self
            .pool
            .execute(sql, &[&now])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(affected as u64)
    }
}
