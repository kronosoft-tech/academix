//! Control Plane Repository (stub for Phase 1).
//!
//! The full implementation lives in Phase 2 once the control plane Turso DB
//! is bootstrapped. For now, we define the core types so that
//! ConnectionManager, MemoryBuffer, and FlushTimer can reference them.
//!
//! Phase 2 will replace this with a real Turso-backed implementation.

/// User-to-database mapping stored in the control plane.
///
/// Maps each user account to their isolated Turso database connection details.
#[derive(Debug, Clone)]
pub struct UserDbMapping {
    pub user_id: String,
    pub email: String,
    pub academy_name: String,
    pub db_url: String,
    pub db_token: String,
    pub org: String,
    pub created_at: String,
}

/// Control plane repository (placeholder).
///
/// Maps user accounts to their Turso database connection details.
/// The full implementation in Phase 2 will connect to the `academix-control-plane`
/// Turso database.
///
/// # Phase 2 changes
/// - Add `db: libsql::Database` field
/// - Implement `save_user_db()`
/// - Implement `find_by_email()`
/// - Implement `find_by_user_id()`
/// - Implement `list_all_databases()`
/// - Seed superadmin on startup
#[allow(dead_code)]
pub struct ControlPlaneRepository {}

#[allow(dead_code)]
impl ControlPlaneRepository {
    /// Create a new control plane repository (stub).
    ///
    /// In Phase 2, this will connect to the `academix-control-plane` Turso DB
    /// using libsql with the URL and token from environment variables.
    pub fn new() -> Self {
        Self {}
    }

    /// Find a user-to-database mapping by email address.
    ///
    /// # Phase 2
    /// Will query the `user_databases` table in the control plane Turso DB.
    pub async fn find_by_email(&self, _email: &str) -> Result<Option<UserDbMapping>, String> {
        // STUB: Phase 2 implementation will query Turso DB
        Err("Control plane not yet implemented (Phase 2)".to_string())
    }

    /// Find a user-to-database mapping by user_id.
    ///
    /// # Phase 2
    /// Will query the `user_databases` table in the control plane Turso DB.
    pub async fn find_by_user_id(
        &self,
        _user_id: &str,
    ) -> Result<Option<UserDbMapping>, String> {
        // STUB: Phase 2 implementation will query Turso DB
        Err("Control plane not yet implemented (Phase 2)".to_string())
    }

    /// Save a user-to-database mapping.
    ///
    /// # Phase 2
    /// Will insert into the `user_databases` table in the control plane Turso DB.
    pub async fn save_user_db(&self, _mapping: &UserDbMapping) -> Result<(), String> {
        // STUB: Phase 2 implementation will insert into Turso DB
        Err("Control plane not yet implemented (Phase 2)".to_string())
    }

    /// List all user-to-database mappings (for superadmin).
    ///
    /// # Phase 2
    /// Will query all rows from the `user_databases` table.
    pub async fn list_all_databases(&self) -> Result<Vec<UserDbMapping>, String> {
        // STUB: Phase 2 implementation will query Turso DB
        Err("Control plane not yet implemented (Phase 2)".to_string())
    }
}

impl Default for ControlPlaneRepository {
    fn default() -> Self {
        Self::new()
    }
}
