//! In-memory write buffer with immediate async flush.
//!
//! All CRUD operations write here first. A background task flushes to Turso
//! immediately after each write. On failure, operations stay buffered and
//! are retried with exponential backoff.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Notify;

/// A buffered database operation
#[derive(Debug, Clone)]
pub enum BufferedOperation {
    /// Insert a new row into a table
    Insert {
        table: String,
        data: HashMap<String, String>,
    },
    /// Update an existing row by id
    Update {
        table: String,
        id: String,
        data: HashMap<String, String>,
    },
    /// Delete a row by id
    Delete { table: String, id: String },
}

/// Cached read result for a table query or entity lookup.
#[derive(Debug, Clone)]
pub enum ReadCacheEntry {
    /// Full-table list query result (e.g., find_all)
    List(Vec<HashMap<String, String>>),
    /// Single entity lookup result (e.g., find_by_id)
    Entity(Option<HashMap<String, String>>),
}

/// Tables that support read caching.
pub const CACHEABLE_TABLES: &[&str] = &[
    "students",
    "courses",
    "groups_table",
    "payments",
    "attendance",
    "accounting_entries",
    "users",
];

/// Thread-safe in-memory write buffer.
///
/// All writes go through this buffer. Reads check buffer first, then Turso.
/// A background task is notified immediately on each write and flushes
/// operations to Turso asynchronously. On flush failure, operations remain
/// in the buffer and are retried with exponential backoff.
pub struct MemoryBuffer {
    /// Pending writes grouped by user_id
    pending_writes: HashMap<String, Vec<BufferedOperation>>,
    /// Read cache: user_id → cache_key → entry
    /// cache_key format: "{table}" for lists, "{table}:{id}" for entities
    read_cache: HashMap<String, HashMap<String, ReadCacheEntry>>,
    /// Timestamp of the last write operation
    last_write_at: Instant,
    /// Notify handle — signals the flush loop to wake immediately
    flush_notify: Arc<Notify>,
}

impl MemoryBuffer {
    /// Create a new empty MemoryBuffer.
    pub fn new() -> Self {
        Self {
            pending_writes: HashMap::new(),
            read_cache: HashMap::new(),
            last_write_at: Instant::now(),
            flush_notify: Arc::new(Notify::new()),
        }
    }

    /// Get a clone of the flush notify handle (for the flush loop).
    pub fn flush_notify(&self) -> Arc<Notify> {
        Arc::clone(&self.flush_notify)
    }

    /// Buffer a write operation (create/update/delete).
    /// Immediately signals the background flush task to wake up.
    pub fn buffer_write(&mut self, user_id: &str, op: BufferedOperation) {
        // Extract table name before pushing
        let table = match &op {
            BufferedOperation::Insert { table, .. } => table.clone(),
            BufferedOperation::Update { table, .. } => table.clone(),
            BufferedOperation::Delete { table, .. } => table.clone(),
        };

        println!(
            "[BUFFER] Write for user='{}', op={}",
            user_id,
            match &op {
                BufferedOperation::Insert { table, .. } => format!("INSERT {}", table),
                BufferedOperation::Update { table, id, .. } =>
                    format!("UPDATE {} id={}", table, id),
                BufferedOperation::Delete { table, id } => format!("DELETE {} id={}", table, id),
            }
        );

        self.pending_writes
            .entry(user_id.to_string())
            .or_default()
            .push(op);
        self.last_write_at = Instant::now();

        // Invalidate read cache for the affected table
        self.invalidate_table_cache(user_id, &table);

        // Signal flush loop to wake immediately
        self.flush_notify.notify_one();
    }

    /// Get time elapsed since last write.
    pub fn idle_duration(&self) -> std::time::Duration {
        self.last_write_at.elapsed()
    }

    /// Check if there are pending writes for a specific user.
    pub fn has_pending_writes(&self, user_id: &str) -> bool {
        self.pending_writes
            .get(user_id)
            .map_or(false, |writes| !writes.is_empty())
    }

    /// Get all user IDs that have pending writes.
    pub fn users_with_pending_writes(&self) -> Vec<String> {
        self.pending_writes
            .iter()
            .filter(|(_, ops)| !ops.is_empty())
            .map(|(uid, _)| uid.clone())
            .collect()
    }

    /// Take pending writes for a user (clears them from buffer).
    ///
    /// Returns the buffered operations so the caller can flush them to Turso.
    pub fn take_pending_writes(&mut self, user_id: &str) -> Vec<BufferedOperation> {
        self.pending_writes.remove(user_id).unwrap_or_default()
    }

    /// Re-enqueue operations that failed to flush (puts them back at the front).
    pub fn requeue_writes(&mut self, user_id: &str, ops: Vec<BufferedOperation>) {
        if ops.is_empty() {
            return;
        }
        let entry = self.pending_writes.entry(user_id.to_string()).or_default();
        // Put failed ops at the front so they're retried first
        let mut combined = ops;
        combined.append(entry);
        *entry = combined;
    }

    /// Total number of pending operations across all users.
    pub fn pending_count(&self) -> usize {
        self.pending_writes.values().map(|v| v.len()).sum()
    }

    /// Find a session by token in pending writes.
    ///
    /// Searches through all pending Insert operations for the "sessions" table
    /// and returns the user_id and session data if the token matches.
    /// Phase 4: Basic linear scan. Phase 5 will add a proper token→user_id index.
    pub fn find_session_by_token(&self, token: &str) -> Option<(String, HashMap<String, String>)> {
        for (user_id, ops) in &self.pending_writes {
            for op in ops {
                if let BufferedOperation::Insert { table, data } = op {
                    if table == "sessions" {
                        if let Some(t) = data.get("token") {
                            if t == token {
                                return Some((user_id.clone(), data.clone()));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Reset the idle timer (call after flush completes).
    pub fn reset_timer(&mut self) {
        self.last_write_at = Instant::now();
    }

    /// Find a pending Insert operation for a given table+id combination.
    ///
    /// Matches by the `id` key in the Insert data HashMap.
    /// Returns `None` if no matching pending insert exists.
    pub fn find_pending_insert(
        &self,
        user_id: &str,
        table: &str,
        id: &str,
    ) -> Option<&BufferedOperation> {
        self.pending_writes.get(user_id).and_then(|ops| {
            ops.iter().find(|op| match op {
                BufferedOperation::Insert { table: t, data } => {
                    t == table && data.get("id").map(|v| v.as_str()) == Some(id)
                }
                _ => false,
            })
        })
    }

    /// Find a pending Update operation for a given table+id.
    pub fn find_pending_update(
        &self,
        user_id: &str,
        table: &str,
        id: &str,
    ) -> Option<&BufferedOperation> {
        self.pending_writes.get(user_id).and_then(|ops| {
            ops.iter().find(|op| match op {
                BufferedOperation::Update {
                    table: t, id: i, ..
                } => t == table && i == id,
                _ => false,
            })
        })
    }

    /// Check if a pending Delete exists for a given table+id.
    pub fn has_pending_delete(&self, user_id: &str, table: &str, id: &str) -> bool {
        self.pending_writes.get(user_id).map_or(false, |ops| {
            ops.iter().any(|op| match op {
                BufferedOperation::Delete { table: t, id: i } => t == table && i == id,
                _ => false,
            })
        })
    }

    /// Scan all pending Insert operations for a specific table (for listing/aggregation).
    pub fn scan_pending_inserts<'a>(
        &'a self,
        user_id: &str,
        table: &str,
    ) -> Vec<&'a BufferedOperation> {
        self.pending_writes.get(user_id).map_or(Vec::new(), |ops| {
            ops.iter()
                .filter(|op| matches!(op, BufferedOperation::Insert { table: t, .. } if t == table))
                .collect()
        })
    }

    /// Scan all pending Update operations for a specific table.
    pub fn scan_pending_updates<'a>(
        &'a self,
        user_id: &str,
        table: &str,
    ) -> Vec<&'a BufferedOperation> {
        self.pending_writes.get(user_id).map_or(Vec::new(), |ops| {
            ops.iter()
                .filter(|op| matches!(op, BufferedOperation::Update { table: t, .. } if t == table))
                .collect()
        })
    }

    /// Scan all pending Delete operations for a specific table.
    pub fn scan_pending_deletes<'a>(
        &'a self,
        user_id: &str,
        table: &str,
    ) -> Vec<&'a BufferedOperation> {
        self.pending_writes.get(user_id).map_or(Vec::new(), |ops| {
            ops.iter()
                .filter(|op| matches!(op, BufferedOperation::Delete { table: t, .. } if t == table))
                .collect()
        })
    }

    /// Get cached list result for a table.
    /// Returns None on cache miss.
    pub fn get_cached_list(
        &self,
        user_id: &str,
        table: &str,
    ) -> Option<&Vec<HashMap<String, String>>> {
        if !CACHEABLE_TABLES.contains(&table) {
            return None;
        }
        self.read_cache
            .get(user_id)
            .and_then(|entries| entries.get(table))
            .and_then(|entry| match entry {
                ReadCacheEntry::List(data) => Some(data),
                _ => None,
            })
    }

    /// Store a list result in the cache.
    pub fn set_cached_list(
        &mut self,
        user_id: &str,
        table: &str,
        data: Vec<HashMap<String, String>>,
    ) {
        if !CACHEABLE_TABLES.contains(&table) {
            return;
        }
        self.read_cache
            .entry(user_id.to_string())
            .or_default()
            .insert(table.to_string(), ReadCacheEntry::List(data));
    }

    /// Get a cached single-entity lookup result.
    /// Returns None on cache miss.
    pub fn get_cached_entity(
        &self,
        user_id: &str,
        table: &str,
        id: &str,
    ) -> Option<&Option<HashMap<String, String>>> {
        if !CACHEABLE_TABLES.contains(&table) {
            return None;
        }
        let key = format!("{}:{}", table, id);
        self.read_cache
            .get(user_id)
            .and_then(|entries| entries.get(&key))
            .and_then(|entry| match entry {
                ReadCacheEntry::Entity(data) => Some(data),
                _ => None,
            })
    }

    /// Store a single-entity lookup result in the cache.
    pub fn set_cached_entity(
        &mut self,
        user_id: &str,
        table: &str,
        id: &str,
        data: Option<HashMap<String, String>>,
    ) {
        if !CACHEABLE_TABLES.contains(&table) {
            return;
        }
        let key = format!("{}:{}", table, id);
        self.read_cache
            .entry(user_id.to_string())
            .or_default()
            .insert(key, ReadCacheEntry::Entity(data));
    }

    /// Invalidate all cached entries for a specific table.
    /// Called inside `buffer_write` after pushing the operation.
    pub fn invalidate_table_cache(&mut self, user_id: &str, table: &str) {
        if let Some(entries) = self.read_cache.get_mut(user_id) {
            // Remove the list entry for this table
            entries.remove(table);
            // Remove all entity entries for this table (keys starting with "{table}:")
            let prefix = format!("{}:", table);
            entries.retain(|key, _| !key.starts_with(&prefix));
        }
    }

    /// Clear all read cache entries for a user (on session end/disconnect).
    pub fn clear_user_cache(&mut self, user_id: &str) {
        self.read_cache.remove(user_id);
    }

    /// Clear all read cache entries (on full reset).
    pub fn clear_all_read_cache(&mut self) {
        self.read_cache.clear();
    }
}

impl Default for MemoryBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_is_empty() {
        let buffer = MemoryBuffer::new();
        assert_eq!(buffer.pending_count(), 0);
        assert!(buffer.users_with_pending_writes().is_empty());
    }

    #[test]
    fn test_buffer_write_adds_operation() {
        let mut buffer = MemoryBuffer::new();
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Alice".to_string());

        buffer.buffer_write(
            "user-1",
            BufferedOperation::Insert {
                table: "users".to_string(),
                data,
            },
        );

        assert_eq!(buffer.pending_count(), 1);
        assert!(buffer.has_pending_writes("user-1"));
    }

    #[test]
    fn test_take_pending_writes_clears_buffer() {
        let mut buffer = MemoryBuffer::new();
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Bob".to_string());

        buffer.buffer_write(
            "user-1",
            BufferedOperation::Insert {
                table: "users".to_string(),
                data,
            },
        );

        let ops = buffer.take_pending_writes("user-1");
        assert_eq!(ops.len(), 1);
        assert!(!buffer.has_pending_writes("user-1"));
        assert_eq!(buffer.pending_count(), 0);
    }

    #[test]
    fn test_users_with_pending_writes() {
        let mut buffer = MemoryBuffer::new();
        buffer.buffer_write(
            "user-1",
            BufferedOperation::Delete {
                table: "students".to_string(),
                id: "s1".to_string(),
            },
        );
        buffer.buffer_write(
            "user-2",
            BufferedOperation::Delete {
                table: "courses".to_string(),
                id: "c1".to_string(),
            },
        );

        let users = buffer.users_with_pending_writes();
        assert_eq!(users.len(), 2);
        assert!(users.contains(&"user-1".to_string()));
        assert!(users.contains(&"user-2".to_string()));
    }

    #[test]
    fn test_idle_duration_increases() {
        let buffer = MemoryBuffer::new();
        let idle = buffer.idle_duration();
        assert!(idle.as_secs() == 0 || idle.as_secs() > 0);
        // After a tiny sleep we should see a non-zero duration
        std::thread::sleep(std::time::Duration::from_millis(5));
        let idle2 = buffer.idle_duration();
        assert!(idle2 >= std::time::Duration::from_millis(5));
    }

    #[test]
    fn test_write_resets_timer() {
        let mut buffer = MemoryBuffer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let before = buffer.idle_duration();

        buffer.buffer_write(
            "user-1",
            BufferedOperation::Delete {
                table: "x".to_string(),
                id: "1".to_string(),
            },
        );

        let after = buffer.idle_duration();
        assert!(after < before);
    }

    #[test]
    fn test_set_and_get_cached_list() {
        let mut buf = MemoryBuffer::new();
        let data = vec![{
            let mut m = HashMap::new();
            m.insert("id".to_string(), "1".to_string());
            m.insert("name".to_string(), "Test".to_string());
            m
        }];
        buf.set_cached_list("user1", "students", data.clone());
        let cached = buf.get_cached_list("user1", "students");
        assert_eq!(cached, Some(&data));
    }

    #[test]
    fn test_set_and_get_cached_entity() {
        let mut buf = MemoryBuffer::new();
        let mut entity = HashMap::new();
        entity.insert("id".to_string(), "abc".to_string());
        buf.set_cached_entity("user1", "students", "abc", Some(entity.clone()));
        let cached = buf.get_cached_entity("user1", "students", "abc");
        assert_eq!(cached, Some(&Some(entity)));
    }

    #[test]
    fn test_cache_miss_returns_none() {
        let buf = MemoryBuffer::new();
        assert_eq!(buf.get_cached_list("user1", "students"), None);
        assert_eq!(buf.get_cached_entity("user1", "students", "123"), None);
    }

    #[test]
    fn test_invalidate_table_clears_list_and_entities() {
        let mut buf = MemoryBuffer::new();
        buf.set_cached_list("user1", "students", vec![HashMap::new()]);
        buf.set_cached_entity("user1", "students", "id1", Some(HashMap::new()));
        buf.set_cached_entity("user1", "students", "id2", Some(HashMap::new()));

        buf.invalidate_table_cache("user1", "students");

        assert_eq!(buf.get_cached_list("user1", "students"), None);
        assert_eq!(buf.get_cached_entity("user1", "students", "id1"), None);
        assert_eq!(buf.get_cached_entity("user1", "students", "id2"), None);
    }

    #[test]
    fn test_invalidate_preserves_other_tables() {
        let mut buf = MemoryBuffer::new();
        buf.set_cached_list("user1", "students", vec![HashMap::new()]);
        buf.set_cached_list("user1", "courses", vec![HashMap::new()]);

        buf.invalidate_table_cache("user1", "students");

        assert_eq!(buf.get_cached_list("user1", "students"), None);
        assert!(buf.get_cached_list("user1", "courses").is_some());
    }

    #[test]
    fn test_buffer_write_invalidates_cache() {
        let mut buf = MemoryBuffer::new();
        buf.set_cached_list("user1", "students", vec![HashMap::new()]);

        let mut data = HashMap::new();
        data.insert("id".to_string(), "new-id".to_string());
        buf.buffer_write(
            "user1",
            BufferedOperation::Insert {
                table: "students".to_string(),
                data,
            },
        );

        assert_eq!(buf.get_cached_list("user1", "students"), None);
    }

    #[test]
    fn test_non_cacheable_table_bypasses_cache() {
        let mut buf = MemoryBuffer::new();
        buf.set_cached_list("user1", "sessions", vec![HashMap::new()]);
        assert_eq!(buf.get_cached_list("user1", "sessions"), None);

        buf.set_cached_entity("user1", "sessions", "id1", Some(HashMap::new()));
        assert_eq!(buf.get_cached_entity("user1", "sessions", "id1"), None);
    }

    #[test]
    fn test_clear_user_cache() {
        let mut buf = MemoryBuffer::new();
        buf.set_cached_list("user1", "students", vec![HashMap::new()]);
        buf.set_cached_list("user2", "students", vec![HashMap::new()]);

        buf.clear_user_cache("user1");

        assert_eq!(buf.get_cached_list("user1", "students"), None);
        assert!(buf.get_cached_list("user2", "students").is_some());
    }

    #[test]
    fn test_clear_all_read_cache() {
        let mut buf = MemoryBuffer::new();
        buf.set_cached_list("user1", "students", vec![HashMap::new()]);
        buf.set_cached_list("user2", "courses", vec![HashMap::new()]);

        buf.clear_all_read_cache();

        assert_eq!(buf.get_cached_list("user1", "students"), None);
        assert_eq!(buf.get_cached_list("user2", "courses"), None);
    }
}
