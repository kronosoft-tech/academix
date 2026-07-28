//! Local libSQL Database Module
//!
//! Provides a global local database instance using libsql (replaces rusqlite).
//! Initialized once at app startup and reused across repos.

use std::sync::OnceLock;

static LOCAL_DB: OnceLock<libsql::Database> = OnceLock::new();

/// Initialize the local database (call once at app startup).
pub fn init(db: libsql::Database) {
    LOCAL_DB.set(db).ok();
}

/// Get a reference to the local database.
pub fn get_db() -> &'static libsql::Database {
    LOCAL_DB.get().expect("Local DB not initialized. Call local_db::init() first.")
}
