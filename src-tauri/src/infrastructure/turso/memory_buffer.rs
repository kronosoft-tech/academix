//! In-memory write buffer with inactivity-based flush.
//!
//! All CRUD operations write here first. A background timer flushes to Turso
//! after 15 minutes without any write activity.

use std::collections::HashMap;
use std::time::Instant;

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
    Delete {
        table: String,
        id: String,
    },
}

/// Generic cached entity (serialized as JSON)
#[derive(Debug, Clone)]
pub struct CachedEntity {
    pub id: String,
    pub data: HashMap<String, String>,
}

/// Thread-safe in-memory write buffer.
///
/// All writes go through this buffer. Reads check buffer first, then Turso.
/// A background timer monitors write activity and flushes after 15 minutes
/// of inactivity.
pub struct MemoryBuffer {
    /// Pending writes grouped by user_id
    pending_writes: HashMap<String, Vec<BufferedOperation>>,
    /// Cached read results (user_id → entity_key → CachedEntity)
    cached_entities: HashMap<String, HashMap<String, CachedEntity>>,
    /// Timestamp of the last write operation
    last_write_at: Instant,
}

impl MemoryBuffer {
    /// Create a new empty MemoryBuffer.
    pub fn new() -> Self {
        Self {
            pending_writes: HashMap::new(),
            cached_entities: HashMap::new(),
            last_write_at: Instant::now(),
        }
    }

    /// Buffer a write operation (create/update/delete).
    /// Resets the idle timer on each write.
    pub fn buffer_write(&mut self, user_id: &str, op: BufferedOperation) {
        self.pending_writes
            .entry(user_id.to_string())
            .or_default()
            .push(op);
        self.last_write_at = Instant::now();
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

    /// Cache an entity for read-through.
    pub fn cache_entity(&mut self, user_id: &str, key: &str, entity: CachedEntity) {
        self.cached_entities
            .entry(user_id.to_string())
            .or_default()
            .insert(key.to_string(), entity);
    }

    /// Get a cached entity by user_id and key.
    pub fn get_cached(&self, user_id: &str, key: &str) -> Option<&CachedEntity> {
        self.cached_entities
            .get(user_id)
            .and_then(|entities| entities.get(key))
    }

    /// Clear all cached entities for a user (after flush).
    pub fn clear_cache(&mut self, user_id: &str) {
        if let Some(entities) = self.cached_entities.get_mut(user_id) {
            entities.clear();
        }
    }

    /// Clear all cached entities (after full flush).
    pub fn clear_all_caches(&mut self) {
        self.cached_entities.clear();
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
    pub fn find_pending_insert(&self, user_id: &str, table: &str, id: &str) -> Option<&BufferedOperation> {
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
    pub fn find_pending_update(&self, user_id: &str, table: &str, id: &str) -> Option<&BufferedOperation> {
        self.pending_writes.get(user_id).and_then(|ops| {
            ops.iter().find(|op| match op {
                BufferedOperation::Update { table: t, id: i, .. } => t == table && i == id,
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
    pub fn scan_pending_inserts<'a>(&'a self, user_id: &str, table: &str) -> Vec<&'a BufferedOperation> {
        self.pending_writes.get(user_id).map_or(Vec::new(), |ops| {
            ops.iter()
                .filter(|op| matches!(op, BufferedOperation::Insert { table: t, .. } if t == table))
                .collect()
        })
    }

    /// Scan all pending Update operations for a specific table.
    pub fn scan_pending_updates<'a>(&'a self, user_id: &str, table: &str) -> Vec<&'a BufferedOperation> {
        self.pending_writes.get(user_id).map_or(Vec::new(), |ops| {
            ops.iter()
                .filter(|op| matches!(op, BufferedOperation::Update { table: t, .. } if t == table))
                .collect()
        })
    }

    /// Scan all pending Delete operations for a specific table.
    pub fn scan_pending_deletes<'a>(&'a self, user_id: &str, table: &str) -> Vec<&'a BufferedOperation> {
        self.pending_writes.get(user_id).map_or(Vec::new(), |ops| {
            ops.iter()
                .filter(|op| matches!(op, BufferedOperation::Delete { table: t, .. } if t == table))
                .collect()
        })
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
    fn test_cache_entity_and_retrieve() {
        let mut buffer = MemoryBuffer::new();
        let entity = CachedEntity {
            id: "u1".to_string(),
            data: HashMap::from([("name".to_string(), "Alice".to_string())]),
        };

        buffer.cache_entity("user-1", "profile", entity.clone());
        let cached = buffer.get_cached("user-1", "profile");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id, "u1");
    }

    #[test]
    fn test_clear_cache() {
        let mut buffer = MemoryBuffer::new();
        let entity = CachedEntity {
            id: "u1".to_string(),
            data: HashMap::new(),
        };

        buffer.cache_entity("user-1", "key1", entity);
        assert!(buffer.get_cached("user-1", "key1").is_some());

        buffer.clear_cache("user-1");
        assert!(buffer.get_cached("user-1", "key1").is_none());
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
}
