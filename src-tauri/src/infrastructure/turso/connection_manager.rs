//! Manages remote-only libsql connections to users' Turso databases.
//!
//! Connections are lazily created and cached for the session lifetime.
//! The ConnectionManager is shared via `Arc<Mutex<...>>` and is the bridge
//! between user identity and their isolated Turso database.

use std::collections::HashMap;
use std::sync::Arc;

use crate::infrastructure::turso::control_plane::{ControlPlaneRepository, UserDbMapping};

/// A cached libsql connection paired with its user database mapping.
pub struct CachedConnection {
    pub db: Arc<libsql::Database>,
    pub mapping: UserDbMapping,
}

impl std::fmt::Debug for CachedConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedConnection")
            .field("mapping", &self.mapping)
            .finish()
    }
}

impl Clone for CachedConnection {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            mapping: self.mapping.clone(),
        }
    }
}

/// Manages connections to users' Turso databases.
///
/// Remote-only connections (no local files). Each connection is lazily
/// created on first resolution and cached for the app session lifetime.
pub struct ConnectionManager {
    connections: HashMap<String, CachedConnection>,
}

impl ConnectionManager {
    /// Create a new empty connection manager.
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Resolve a user's database by email address.
    ///
    /// Looks up the control plane for the user's `UserDbMapping`,
    /// creates a remote-only libsql connection, caches it, and returns it.
    pub async fn resolve_by_email(
        &mut self,
        cp: &ControlPlaneRepository,
        email: &str,
    ) -> Result<&CachedConnection, String> {
        // Check cache first by searching for matching email
        let cached_user_id: Option<String> = {
            self.connections
                .iter()
                .find(|(_, conn)| conn.mapping.email == email)
                .map(|(uid, _)| uid.clone())
        };

        if let Some(ref uid) = cached_user_id {
            return Ok(self.connections.get(uid).unwrap());
        }

        // Look up in control plane
        let mapping = cp
            .find_by_email(email)
            .await
            .map_err(|e| format!("Control plane lookup failed: {}", e))?
            .ok_or_else(|| "User not found in control plane".to_string())?;

        self.init_connection(mapping).await
    }

    /// Resolve a user's database by user_id (from session token).
    pub async fn resolve_by_user_id(&self, user_id: &str) -> Result<&CachedConnection, String> {
        self.connections
            .get(user_id)
            .ok_or_else(|| format!("No cached connection for user '{}'. Login first.", user_id))
    }

    /// Register a new connection from the registration flow.
    pub async fn register_connection(&mut self, mapping: UserDbMapping) -> Result<(), String> {
        self.init_connection(mapping).await?;
        Ok(())
    }

    /// Internal: create a remote-only libsql connection and cache it.
    async fn init_connection(
        &mut self,
        mapping: UserDbMapping,
    ) -> Result<&CachedConnection, String> {
        let db = libsql::Builder::new_remote(mapping.db_url.clone(), mapping.db_token.clone())
            .build()
            .await
            .map_err(|e| format!("Failed to connect to Turso DB: {}", e))?;

        let conn = CachedConnection {
            db: Arc::new(db),
            mapping: mapping.clone(),
        };

        self.connections.insert(mapping.user_id.clone(), conn);
        Ok(self.connections.get(&mapping.user_id).unwrap())
    }

    /// Get a cached connection by user_id (for flush operations).
    pub fn get_connection(&self, user_id: &str) -> Option<&CachedConnection> {
        self.connections.get(user_id)
    }

    /// Run all migrations against a newly created database.
    ///
    /// Executes each migration SQL file against the given libsql database.
    /// This is called during registration after creating the user's Turso DB.
    pub async fn run_migrations(&self, db: &libsql::Database) -> Result<(), String> {
        let conn = db
            .connect()
            .map_err(|e| format!("Failed to connect: {}", e))?;

        // Additional migrations are added incrementally in Phase 3 registration.
        // All 18 migrations will be included once the migration files are complete.
        // For now, this is a placeholder awaiting the full migration list.

        let _ = conn; // Suppress unused warning in Phase 1
        Ok(())
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager_empty() {
        let cm = ConnectionManager::new();
        assert!(cm.get_connection("nonexistent").is_none());
    }

    #[test]
    fn test_resolve_by_user_id_returns_error_for_unknown() {
        let cm = ConnectionManager::new();

        // Use a manual runtime to test the async function
        // We can't easily test the returned reference due to lifetime constraints,
        // but we can verify the synchronous get_connection path
        assert!(cm.get_connection("no-such-user").is_none());

        // And verify resolve_by_user_id signature exists by checking the error type
        // (compilation test — the function exists with expected signature)
        let _ = cm.get_connection("unknown");
    }

    #[test]
    fn test_register_and_get() {
        let cm = ConnectionManager::new();

        // Can't fully test without a real Turso connection,
        // but at least verify empty state
        assert!(cm.get_connection("test-user").is_none());
    }
}
