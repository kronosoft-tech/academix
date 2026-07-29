# Requirements Document

## Introduction

Add a read cache layer to the existing `MemoryBuffer` struct that eliminates redundant network calls to Turso for repeated reads. The cache stores query results in RAM after the first read, serves subsequent reads from memory, and invalidates entries when writes touch the same table. This improves perceived app responsiveness by removing 100-300ms network latency on repeated data access within a session.

## Glossary

- **Read_Cache**: An in-memory HashMap structure within `MemoryBuffer` that stores previously fetched query results, keyed by user_id and table name, to avoid redundant Turso network calls.
- **MemoryBuffer**: The existing Rust struct (`src-tauri/src/infrastructure/turso/memory_buffer.rs`) that buffers write operations for async flush to Turso. The Read_Cache will be added inside this struct.
- **Cache_Entry**: A stored result for a specific cache key, containing either a full table listing (Vec of serialized entities) or a single entity lookup result.
- **Table_Cache**: The portion of the Read_Cache scoped to a single table name for a given user (e.g., all cached reads for "students" belonging to user X).
- **Cache_Invalidation**: The process of clearing all Cache_Entries for a given table when a write operation (`buffer_write`) targets that table.
- **Cache_Key**: A composite identifier used to look up a Cache_Entry, composed of the table name plus an optional entity ID (for single-entity lookups) or a sentinel value (for full-table list queries).
- **Repository**: A MemoryBacked repository implementation (e.g., `MemoryBackedStudentRepository`) that reads from Turso and merges with pending buffer writes.
- **Turso**: The remote libSQL database service that stores the canonical data.

## Requirements

### Requirement 1: Cache Storage Structure

**User Story:** As a developer, I want a structured in-memory cache inside MemoryBuffer, so that read results can be stored and retrieved efficiently without modifying the existing write buffer logic.

#### Acceptance Criteria

1. THE Read_Cache SHALL store Cache_Entries in a HashMap keyed by user_id, then by table name, then by Cache_Key
2. THE Read_Cache SHALL support two Cache_Entry types: a full-table list result and a single-entity lookup result
3. THE Read_Cache SHALL be isolated per user_id so that one user's cached data is never visible to another user

### Requirement 2: Cache Population on First Read

**User Story:** As a user navigating the app, I want the first read from Turso to populate the cache, so that subsequent reads of the same data are instant.

#### Acceptance Criteria

1. WHEN a Repository performs a full-table list query (e.g., `find_all`) and the Table_Cache for that table is empty, THE Repository SHALL query Turso, store the result in the Read_Cache, and return the result
2. WHEN a Repository performs a single-entity lookup (e.g., `find_by_id`) and no Cache_Entry exists for that entity, THE Repository SHALL query Turso, store the result in the Read_Cache, and return the result
3. WHEN a Repository performs a full-table list query and a valid Cache_Entry already exists for that table, THE Repository SHALL return the cached result without making a network call to Turso. IF the Cache_Entry is found to be corrupt or unreadable, THE Repository SHALL discard it, query Turso, and repopulate the cache.
4. WHEN a Repository performs a single-entity lookup and a valid Cache_Entry already exists for that entity, THE Repository SHALL return the cached result without making a network call to Turso

### Requirement 3: Cache Invalidation on Write

**User Story:** As a user making changes to data, I want the cache to be invalidated when I write to a table, so that subsequent reads always reflect my latest changes.

#### Acceptance Criteria

1. WHEN `buffer_write` is called with a BufferedOperation targeting table X, THE MemoryBuffer SHALL clear all Cache_Entries in the Table_Cache for table X for that user_id
2. WHEN `buffer_write` is called with a BufferedOperation targeting table X, THE MemoryBuffer SHALL preserve Cache_Entries for all other tables unrelated to table X
3. WHEN the cache is invalidated for a table, THE next read for that table SHALL query Turso again and repopulate the cache

### Requirement 4: Merge With Pending Writes

**User Story:** As a user who has unsaved changes in the buffer, I want cached reads to still reflect my pending writes, so that the UI always shows the most up-to-date state.

#### Acceptance Criteria

1. WHEN a Repository returns a cached full-table list, THE Repository SHALL merge the cached result with any pending buffer writes (inserts, updates, deletes) for that table before returning
2. WHEN a Repository returns a cached single-entity lookup, THE Repository SHALL check pending buffer writes for that entity and return the buffered version if a more recent write exists
3. IF a pending delete exists for an entity, THEN THE Repository SHALL exclude that entity from the returned result regardless of cache content

### Requirement 5: Per-User Isolation

**User Story:** As a system serving multiple users, I want each user's cache to be completely independent, so that there is no data leakage between users.

#### Acceptance Criteria

1. THE Read_Cache SHALL key all entries by user_id as the top-level key
2. WHEN a user's session ends or disconnects, THE Read_Cache SHALL clear all Cache_Entries for that user_id only, using the user_id as the sole key for deletion to prevent accidentally affecting other users' data
3. WHEN `clear_cache` is called for a specific user_id, THE Read_Cache SHALL remove only that user's entries and preserve all other users' Cache_Entries

### Requirement 6: Table Coverage

**User Story:** As a developer integrating the cache, I want all main domain tables to be cached, so that the performance benefit applies across the entire application.

#### Acceptance Criteria

1. THE Read_Cache SHALL support caching for the following tables: students, courses, groups_table, payments, attendance, accounting_entries, users
2. WHEN a table not in the supported list is queried, THE Repository SHALL always query Turso directly and SHALL NOT read from or write to the Read_Cache for that table, even if a cache entry exists

### Requirement 7: Session Lifecycle

**User Story:** As a user restarting the app, I want a clean cache on each launch, so that stale data from previous sessions never persists.

#### Acceptance Criteria

1. WHEN the application starts, THE Read_Cache SHALL be empty for all users
2. THE Read_Cache SHALL reside only in RAM with no disk persistence
3. WHEN the MemoryBuffer struct is created via `new()`, THE Read_Cache SHALL be initialized as an empty structure
