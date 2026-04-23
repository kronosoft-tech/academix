//! SQLite Connection Pool
//!
//! Thread-safe SQLite connection wrapper using rusqlite.

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
        // Using WAL (Write-Ahead Logging) mode - optimal for modern apps
        // WAL ensures data persistence and is recommended by SQLite maintainers
        // SYNCHRONOUS = FULL ensures fsync() after every commit for guaranteed persistence
        // cache_size in MB (positive = pages, negative = KB), page_size for alignment
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA cache_size = -64000; PRAGMA page_size = 4096; PRAGMA busy_timeout = 5000;",
        )?;
        
        // Initial checkpoint to clear any incomplete WAL
        conn.execute_batch("PRAGMA wal_checkpoint(RESTART);")?;

        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: db_path,
        };

        // Start background checkpoint thread to persist WAL to main DB file every 5 seconds
        let pool_clone = pool.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(5));
                if let Ok(conn) = pool_clone.conn.lock() {
                    // PASSIVE checkpoint: non-blocking, integrates WAL when possible
                    let _ = conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);");
                }
            }
        });

        Ok(pool)
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
        eprintln!("[DB QUERY] Executing: {}", sql);
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, mapper)?;
        let collected: Result<Vec<T>, _> = rows.collect();
        match &collected {
            Ok(items) => eprintln!("[DB QUERY RESULT] {} rows returned", items.len()),
            Err(e) => eprintln!("[DB QUERY ERROR] {}", e),
        }
        collected
    }

    /// Query for multiple rows with dynamic params
    pub fn query_with_vec<T, F>(
        &self,
        sql: &str,
        params: Vec<String>,
        mapper: F,
    ) -> Result<Vec<T>, String>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        self.query(sql, &params_refs, mapper).map_err(|e| e.to_string())
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

    /// Perform a WAL checkpoint to persist data from WAL log to main database file
    /// This integrates data written to the WAL log back into the .db file
    /// PASSIVE mode: doesn't block new transactions, integrates when possible
    pub fn checkpoint_restart(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        eprintln!("[WAL CHECKPOINT] Attempting PASSIVE checkpoint...");
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;
        eprintln!("[WAL CHECKPOINT] Success - data integrated to main DB");
        Ok(())
    }

    /// Force a blocking checkpoint (RESTART mode)
    /// Use only when you know no other transactions are running
    pub fn checkpoint_force(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        eprintln!("[WAL CHECKPOINT FORCE] Attempting RESTART checkpoint (blocking)...");
        conn.execute_batch("PRAGMA wal_checkpoint(RESTART);")?;
        eprintln!("[WAL CHECKPOINT FORCE] Success - all data integrated");
        Ok(())
    }
}
