//! In-Memory Session Repository

use std::collections::HashMap;
use std::sync::RwLock;

use crate::application::ports::SessionRepository;
use crate::domain::entities::Session;
use crate::domain::errors::DomainError;
use chrono::Utc;

/// In-memory session repository implementation
pub struct InMemorySessionRepository {
    sessions: RwLock<HashMap<String, Session>>,
    tokens: RwLock<HashMap<String, String>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            tokens: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionRepository for InMemorySessionRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, DomainError> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(sessions.get(id).cloned())
    }

    fn find_by_token(&self, token: &str) -> Result<Option<Session>, DomainError> {
        let tokens = self
            .tokens
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let sessions = self
            .sessions
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(id) = tokens.get(token) {
            Ok(sessions.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn save(&self, session: &Session) -> Result<(), DomainError> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut tokens = self
            .tokens
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        sessions.insert(session.id.clone(), session.clone());
        tokens.insert(session.token.clone(), session.id.clone());

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(session) = sessions.get(id) {
            let mut tokens = self
                .tokens
                .write()
                .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
            tokens.remove(&session.token);
            sessions.remove(id);
        }

        Ok(())
    }

    fn delete_expired(&self) -> Result<u64, DomainError> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let now = Utc::now();

        let expired_ids: Vec<String> = sessions
            .values()
            .filter(|s| s.expires_at < now)
            .map(|s| s.id.clone())
            .collect();

        let mut tokens = self
            .tokens
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        for id in &expired_ids {
            if let Some(session) = sessions.get(id) {
                tokens.remove(&session.token);
            }
            sessions.remove(id);
        }

        Ok(expired_ids.len() as u64)
    }
}
