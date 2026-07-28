//! Database Module
//!
//! Global database path and connection management.
//! Phase 4: SqlitePool removed — using rusqlite::Connection directly.
//! Phase 5: Will migrate all repositories to libsql (async).

use std::path::PathBuf;
use std::sync::OnceLock;

/// Global database path, set once during app initialization.
static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Set the global database path (called once during initialization).
pub fn set_db_path(path: PathBuf) {
    let _ = DB_PATH.set(path);
}

/// Get the global database path.
pub fn get_db_path() -> PathBuf {
    DB_PATH.get().cloned().unwrap_or_else(|| {
        let app_data = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("academix");
        std::fs::create_dir_all(&app_data).ok();
        app_data.join("academix.db")
    })
}

/// Open a new rusqlite connection to the global database.
pub fn open_connection() -> Result<rusqlite::Connection, String> {
    let path = get_db_path();
    let conn = rusqlite::Connection::open(&path)
        .map_err(|e| format!("Failed to open database connection: {}", e))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; \
         PRAGMA cache_size = -64000; PRAGMA page_size = 4096; PRAGMA busy_timeout = 5000;",
    )
    .map_err(|e| format!("Failed to set pragmas: {}", e))?;
    Ok(conn)
}
