//! SQLite Connection Pool
//!
//! Thread-safe SQLite connection wrapper using rusqlite.

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Thread-safe SQLite connection pool
#[derive(Clone)]
pub struct SqlitePool {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl SqlitePool {
    /// Create a new pool with database file
    pub fn new(db_path: PathBuf) -> Result<Self, rusqlite::Error> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        // Using DELETE journal mode for guaranteed persistence
        // WAL mode can lose data if app crashes before checkpoint
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = DELETE; PRAGMA synchronous = FULL;",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path: db_path,
        })
    }

    /// Get a locked connection reference
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// Get the database path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Execute a query that doesn't return results
    pub fn execute(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<usize, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(sql, params)
    }

    /// Execute a query and return the last inserted row id
    pub fn execute_insert(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<i64, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(sql, params)?;
        Ok(conn.last_insert_rowid())
    }

    /// Query for multiple rows
    pub fn query<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        mapper: F,
    ) -> Result<Vec<T>, rusqlite::Error>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, mapper)?;
        rows.collect()
    }

    /// Query for a single row
    pub fn query_row<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        mapper: F,
    ) -> Result<Option<T>, rusqlite::Error>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = self.conn.lock().unwrap();
        match conn.query_row(sql, params, mapper) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
