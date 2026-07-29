//! Immediate async flush with retry on failure.
//!
//! Replaces the old 15-minute idle timer. Now flushes immediately when
//! notified by MemoryBuffer, with exponential backoff retry on failure.

use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::control_plane::ControlPlaneRepository;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};

/// Maximum retry delay (caps exponential backoff)
const MAX_RETRY_DELAY: Duration = Duration::from_secs(30);

/// Initial retry delay after a flush failure
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Start the background flush loop.
///
/// Waits on a `Notify` signal from `MemoryBuffer::buffer_write()`.
/// Each write triggers an immediate flush attempt. On failure, operations
/// are requeued and retried with exponential backoff.
///
/// Returns a `oneshot::Sender<()>` for graceful shutdown.
pub fn start_flush_timer(
    buffer: Arc<Mutex<MemoryBuffer>>,
    connection_manager: Arc<Mutex<ConnectionManager>>,
    _control_plane: Arc<ControlPlaneRepository>,
) -> oneshot::Sender<()> {
    println!("[FLUSH] start_flush_timer called — spawning background task...");
    let (tx, mut rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        println!("[FLUSH] Spawn started, acquiring buffer lock...");
        // Get the notify handle from the buffer
        let flush_notify = {
            let buf = buffer.lock().await;
            buf.flush_notify()
        };

        println!("[FLUSH] Background flush loop started — waiting for writes...");

        let mut retry_delay = INITIAL_RETRY_DELAY;
        let mut has_failures = false;

        loop {
            tokio::select! {
                biased;

                _ = &mut rx => {
                    // Shutdown signal — flush everything and exit
                    println!("[FLUSH] Shutdown signal received, flushing...");
                    flush_all(&buffer, &connection_manager).await;
                    break;
                }

                _ = flush_notify.notified() => {
                    // A write just happened — flush immediately
                    // Small debounce: wait 5ms to batch rapid consecutive writes
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    println!("[FLUSH] Notify received, flushing...");
                    let success = flush_all(&buffer, &connection_manager).await;
                    if success {
                        retry_delay = INITIAL_RETRY_DELAY;
                        has_failures = false;
                    } else {
                        has_failures = true;
                    }
                }

                _ = tokio::time::sleep(retry_delay), if has_failures => {
                    // Retry failed operations with backoff
                    let has_pending = {
                        let buf = buffer.lock().await;
                        buf.pending_count() > 0
                    };

                    if has_pending {
                        println!("[FLUSH] Retrying failed operations (delay: {:?})...", retry_delay);
                        let success = flush_all(&buffer, &connection_manager).await;
                        if success {
                            retry_delay = INITIAL_RETRY_DELAY;
                            has_failures = false;
                        } else {
                            // Exponential backoff, capped
                            retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
                            has_failures = true;
                        }
                    } else {
                        has_failures = false;
                        retry_delay = INITIAL_RETRY_DELAY;
                    }
                }
            }
        }
    });

    tx
}

/// Flush all pending writes to Turso. Returns true if ALL operations succeeded.
async fn flush_all(
    buffer: &Arc<Mutex<MemoryBuffer>>,
    connection_manager: &Arc<Mutex<ConnectionManager>>,
) -> bool {
    let user_ids: Vec<String> = {
        let buf = buffer.lock().await;
        buf.users_with_pending_writes()
    };

    if user_ids.is_empty() {
        return true;
    }

    let mut all_success = true;
    let mut total_ops = 0;

    for user_id in &user_ids {
        let ops = {
            let mut buf = buffer.lock().await;
            buf.take_pending_writes(user_id)
        };

        if ops.is_empty() {
            continue;
        }

        total_ops += ops.len();

        // Get the user's database connection
        let maybe_db = {
            let cm = connection_manager.lock().await;
            cm.get_connection(user_id).map(|c| c.clone())
        };

        if let Some(conn) = maybe_db {
            let libsql_conn = conn.db.connect();

            match libsql_conn {
                Ok(libsql_conn) => {
                    let mut failed_ops: Vec<BufferedOperation> = Vec::new();

                    for op in ops {
                        if let Err(e) = execute_operation(&libsql_conn, &op).await {
                            println!(
                                "[FLUSH ERROR] Failed to flush operation for user {}: {}",
                                user_id, e
                            );
                            failed_ops.push(op);
                        }
                    }

                    if !failed_ops.is_empty() {
                        // Requeue failed operations for retry
                        let mut buf = buffer.lock().await;
                        buf.requeue_writes(user_id, failed_ops);
                        all_success = false;
                    } else {
                        // All ops succeeded — clear cache for this user
                        let mut buf = buffer.lock().await;
                        buf.clear_user_cache(user_id);
                    }
                }
                Err(e) => {
                    println!(
                        "[FLUSH ERROR] Failed to connect to Turso DB for user {}: {}",
                        user_id, e
                    );
                    // Requeue ALL ops for this user
                    let mut buf = buffer.lock().await;
                    buf.requeue_writes(user_id, ops);
                    all_success = false;
                }
            }
        } else {
            println!(
                "[FLUSH WARNING] No connection found for user '{}' — requeueing operations",
                user_id
            );
            // Requeue — connection might become available later (e.g. after login)
            let mut buf = buffer.lock().await;
            buf.requeue_writes(user_id, ops);
            all_success = false;
        }
    }

    if all_success && total_ops > 0 {
        // Reset timer
        let mut buf = buffer.lock().await;
        buf.reset_timer();
        println!(
            "[FLUSH] Synced {} operations for {} user(s) to Turso",
            total_ops,
            user_ids.len()
        );
    }

    all_success
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
            let cols: Vec<&String> = data.keys().collect();
            let col_strs: Vec<&str> = cols.iter().map(|k| k.as_str()).collect();
            let placeholders: Vec<String> = (0..data.len()).map(|_| "?".to_string()).collect();

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                col_strs.join(", "),
                placeholders.join(", ")
            );

            let values: Vec<String> = cols.iter().map(|k| data[*k].clone()).collect();
            conn.execute(&sql, libsql::params::params_from_iter(values))
                .await
                .map_err(|e| format!("Insert failed for {}: {}", table, e))?;
            Ok(())
        }
        BufferedOperation::Update { table, id, data } => {
            if data.is_empty() {
                return Err("Update with no data".to_string());
            }
            let sets: Vec<String> = data.keys().map(|k| format!("{} = ?", k)).collect();
            let sql = format!("UPDATE {} SET {} WHERE id = ?", table, sets.join(", "));

            let mut values: Vec<String> = data.values().cloned().collect();
            values.push(id.clone());
            conn.execute(&sql, libsql::params::params_from_iter(values))
                .await
                .map_err(|e| format!("Update failed for {}: {}", table, e))?;
            Ok(())
        }
        BufferedOperation::Delete { table, id } => {
            let sql = format!("DELETE FROM {} WHERE id = ?", table);

            let values = vec![id.clone()];
            conn.execute(&sql, libsql::params::params_from_iter(values))
                .await
                .map_err(|e| format!("Delete failed for {}: {}", table, e))?;
            Ok(())
        }
    }
}

/// Flush immediately (called on app close).
///
/// Has a 5-second timeout. If the flush takes longer, remaining operations
/// are logged and the app closes anyway.
pub async fn flush_on_close(
    buffer: Arc<Mutex<MemoryBuffer>>,
    connection_manager: Arc<Mutex<ConnectionManager>>,
    _control_plane: Arc<ControlPlaneRepository>,
) {
    println!("[FLUSH] App closing — flushing pending writes...");
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        flush_all(&buffer, &connection_manager),
    )
    .await;

    match result {
        Ok(true) => println!("[FLUSH] Close flush complete"),
        Ok(false) => {
            println!("[FLUSH] Close flush had errors — some data may retry on next launch")
        }
        Err(_) => println!("[FLUSH] Timeout reached on close flush — some data may be lost"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify the flush_timer module compiles and basic types work.
    #[tokio::test]
    async fn test_flush_timer_types() {
        let buffer = Arc::new(Mutex::new(MemoryBuffer::new()));
        let cm = Arc::new(Mutex::new(ConnectionManager::new()));

        // Verify buffer is accessible through the Arc<Mutex<...>>
        {
            let buf = buffer.lock().await;
            assert_eq!(buf.pending_count(), 0);
        }
        {
            let cm_lock = cm.lock().await;
            assert!(cm_lock.get_connection("nonexistent").is_none());
        }
    }
}
