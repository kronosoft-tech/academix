use async_trait::async_trait;
use crate::application::ports::SessionRepository;
use crate::domain::entities::Session;
use crate::domain::errors::DomainError;
use crate::infrastructure::local_db;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct SqliteSessionRepository;

impl SqliteSessionRepository {
    pub fn new() -> Self {
        Self
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
}

impl Default for SqliteSessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError> {
        let sql = "SELECT id, user_id, token, expires_at, created_at 
                 FROM sessions WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![id]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => {
                let session = Self::row_to_session(&row)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, DomainError> {
        let sql = "SELECT id, user_id, token, expires_at, created_at 
                 FROM sessions WHERE token = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![token]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => {
                let session = Self::row_to_session(&row)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, session: &Session) -> Result<(), DomainError> {
        let sql = "INSERT INTO sessions (id, user_id, token, expires_at, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(
            sql,
            libsql::params![
                session.id.clone(),
                session.user_id.clone(),
                session.token.clone(),
                session.expires_at.to_rfc3339(),
                session.created_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM sessions WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![id])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Session", id));
        }
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64, DomainError> {
        let now = Utc::now().to_rfc3339();
        let sql = "DELETE FROM sessions WHERE expires_at < ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![now])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(affected as u64)
    }
}
