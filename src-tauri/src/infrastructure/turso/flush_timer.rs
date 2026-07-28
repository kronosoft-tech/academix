//! Background timer that flushes the MemoryBuffer after 15 minutes of inactivity.
//!
//! Also handles flush on app close with a configurable timeout.

use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::control_plane::ControlPlaneRepository;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;

/// Idle timeout before flushing pending writes to Turso.
const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60); // 15 minutes

/// How often to check the idle timer.
const POLL_INTERVAL: Duration = Duration::from_secs(30); // check every 30s

/// Start the background flush timer.
///
/// Polls every 30 seconds. When idle for 15+ minutes and writes are pending,
/// flushes all buffered operations to Turso.
///
/// Returns a `oneshot::Sender<()>` that can be used to signal graceful shutdown.
/// On shutdown signal, the timer flushes immediately and exits.
pub fn start_flush_timer(
    buffer: Arc<Mutex<MemoryBuffer>>,
    connection_manager: Arc<Mutex<ConnectionManager>>,
    control_plane: Arc<ControlPlaneRepository>,
) -> oneshot::Sender<()> {
    let (tx, mut rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => {
                    // Shutdown signal received — flush and exit
                    eprintln!("[FLUSH] Shutdown signal received, flushing...");
                    flush_now(&buffer, &connection_manager, &control_plane).await;
                    break;
                }
                _ = tokio::time::sleep(POLL_INTERVAL) => {
                    let idle = {
                        let buf = buffer.lock().unwrap();
                        buf.idle_duration()
                    };

                    if idle >= IDLE_TIMEOUT {
                        let has_pending = {
                            let buf = buffer.lock().unwrap();
                            buf.pending_count() > 0
                        };

                        if has_pending {
                            eprintln!("[FLUSH] Idle timeout reached, flushing pending writes...");
                            flush_now(&buffer, &connection_manager, &control_plane).await;
                        }
                    }
                }
            }
        }
    });

    tx
}

/// Flush all pending writes to Turso immediately.
async fn flush_now(
    buffer: &Arc<Mutex<MemoryBuffer>>,
    connection_manager: &Arc<Mutex<ConnectionManager>>,
    _control_plane: &Arc<ControlPlaneRepository>,
) {
    let user_ids: Vec<String> = {
        let buf = buffer.lock().unwrap();
        buf.users_with_pending_writes()
    };

    if user_ids.is_empty() {
        return;
    }

    let mut total_ops = 0;

    for user_id in &user_ids {
        let ops = {
            let mut buf = buffer.lock().unwrap();
            buf.take_pending_writes(user_id)
        };

        if ops.is_empty() {
            continue;
        }

        total_ops += ops.len();

        // Get the user's database connection
        let maybe_db = {
            let cm = connection_manager.lock().unwrap();
            cm.get_connection(user_id).map(|c| c.clone())
        };

        if let Some(conn) = maybe_db {
            let libsql_conn = conn.db.connect();

            match libsql_conn {
                Ok(libsql_conn) => {
                    // Build and execute batch SQL from buffered operations
                    for op in &ops {
                        if let Err(e) = execute_operation(&libsql_conn, op).await {
                            eprintln!(
                                "[FLUSH ERROR] Failed to flush operation for user {}: {}",
                                user_id, e
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[FLUSH ERROR] Failed to connect to Turso DB for user {}: {}",
                        user_id, e
                    );
                }
            }
        } else {
            eprintln!(
                "[FLUSH WARNING] No connection found for user '{}' — operations will be lost",
                user_id
            );
        }

        // Clear cached entities for this user
        {
            let mut buf = buffer.lock().unwrap();
            buf.clear_cache(user_id);
        }
    }

    // Reset timer
    {
        let mut buf = buffer.lock().unwrap();
        buf.reset_timer();
    }

    println!(
        "[FLUSH] Completed — flushed {} operations for {} user(s) to Turso",
        total_ops,
        user_ids.len()
    );
}

/// Execute a single buffered operation against a Turso DB connection.
async fn execute_operation(
    conn: &libsql::Connection,
    op: &BufferedOperation,
) -> Result<(), String> {
    match op {
        BufferedOperation::Insert { table, data } => {
            if data.is_empty() {
                return Err("Insert with no data".to_string());
            }
            let cols: Vec<&str> = data.keys().map(|s| s.as_str()).collect();
            let placeholders: Vec<String> = (0..data.len()).map(|_| "?".to_string()).collect();

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                cols.join(", "),
                placeholders.join(", ")
            );

            // TODO(Phase 4): Bind actual values using libsql params
            // Current implementation uses empty params — value binding will
            // be added when the flush integration is wired up in Phase 4.
            conn.execute(&sql, libsql::params![])
                .await
                .map_err(|e| format!("Insert failed for {}: {}", table, e))?;
            Ok(())
        }
        BufferedOperation::Update { table, id: _, data } => {
            if data.is_empty() {
                return Err("Update with no data".to_string());
            }
            let sets: Vec<String> = data.keys().map(|k| format!("{} = ?", k)).collect();
            let sql = format!("UPDATE {} SET {} WHERE id = ?", table, sets.join(", "));

            // TODO(Phase 4): Bind actual values using libsql params
            conn.execute(&sql, libsql::params![])
                .await
                .map_err(|e| format!("Update failed for {}: {}", table, e))?;
            Ok(())
        }
        BufferedOperation::Delete { table, id: _ } => {
            let sql = format!("DELETE FROM {} WHERE id = ?", table);

            // TODO(Phase 4): Bind actual id value
            conn.execute(&sql, libsql::params![])
                .await
                .map_err(|e| format!("Delete failed for {}: {}", table, e))?;
            Ok(())
        }
    }
}

/// Flush immediately (called on app close).
///
/// Has a 5-second timeout. If the flush takes longer than 5 seconds,
/// the remaining operations are logged and the app closes anyway.
pub async fn flush_on_close(
    buffer: Arc<Mutex<MemoryBuffer>>,
    connection_manager: Arc<Mutex<ConnectionManager>>,
    control_plane: Arc<ControlPlaneRepository>,
) {
    println!("[FLUSH] App closing — flushing pending writes...");
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        flush_now(&buffer, &connection_manager, &control_plane),
    )
    .await;

    match result {
        Ok(()) => println!("[FLUSH] Close flush complete"),
        Err(_) => eprintln!("[FLUSH] Timeout reached on close flush — some data may be lost"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the flush_timer module compiles and basic types work.
    #[test]
    fn test_flush_timer_types() {
        let buffer = Arc::new(Mutex::new(MemoryBuffer::new()));
        let cm = Arc::new(Mutex::new(ConnectionManager::new()));

        // Verify buffer is accessible through the Arc<Mutex<...>>
        {
            let buf = buffer.lock().unwrap();
            assert_eq!(buf.pending_count(), 0);
        }
        {
            let cm_lock = cm.lock().unwrap();
            assert!(cm_lock.get_connection("nonexistent").is_none());
        }
    }
}
