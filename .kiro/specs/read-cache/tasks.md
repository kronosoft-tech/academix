# Implementation Plan

## Overview

Implement a read cache layer inside the existing `MemoryBuffer` struct to eliminate redundant Turso network calls. The cache stores query results in RAM after the first read, serves subsequent reads from memory, and invalidates entries when writes touch the same table. Implementation touches `memory_buffer.rs` (core cache logic) and all 7 MemoryBacked repository files (cache integration).

## Tasks

- [x] 1. Add ReadCacheEntry enum and replace cached_entities in MemoryBuffer
  Remove `CachedEntity` struct and `cached_entities` field. Add `ReadCacheEntry` enum with `List(Vec<HashMap<String, String>>)` and `Entity(Option<HashMap<String, String>>)` variants. Replace `cached_entities` with `read_cache: HashMap<String, HashMap<String, ReadCacheEntry>>`. Add `CACHEABLE_TABLES` constant. Update `new()` to init empty. Remove old `cache_entity`, `get_cached`, `clear_cache`, `clear_all_caches` methods.
  **Requirements**: REQ-1

- [x] 2. Add cache read/write methods to MemoryBuffer
  Implement `get_cached_list`, `set_cached_list`, `get_cached_entity`, `set_cached_entity` (with CACHEABLE_TABLES check), `invalidate_table_cache` (removes list + entity entries for a table), `clear_user_cache`, and `clear_all_read_cache`.
  **Requirements**: REQ-1, REQ-5, REQ-6

- [x] 3. Integrate cache invalidation into buffer_write
  Extract table name from BufferedOperation before pushing. After pushing to pending_writes, call `self.invalidate_table_cache(user_id, &table)`. Verify notify is still called after invalidation.
  **Requirements**: REQ-3

- [x] 4. Update flush_timer to use clear_user_cache instead of clear_cache
  Replace calls to `buf.clear_cache(user_id)` with `buf.clear_user_cache(user_id)` after successful flush. Verify compile.
  **Requirements**: REQ-5, REQ-7

- [x] 5. Integrate cache into student repository
  Add `row_to_hash_map` helper. Modify `find_all`: check cache first, query Turso on miss, store result, merge pending writes. Modify `find_by_id`: check entity cache, query on miss, store. Run `cargo check`.
  **Requirements**: REQ-2, REQ-4

- [x] 6. Integrate cache into group repository
  Add `row_to_hash_map`. Modify `find_all` and `find_by_id` (via `find_group_internal`) with cache-first pattern. Run `cargo check`.
  **Requirements**: REQ-2, REQ-4

- [x] 7. Integrate cache into course repository
  Add `row_to_hash_map`. Modify `find_all` and `find_by_id` with cache-first pattern. Run `cargo check`.
  **Requirements**: REQ-2, REQ-4

- [x] 8. Integrate cache into payment repository
  Add `row_to_hash_map`. Modify `find_by_student_id`, `find_by_group_id`, and `find_by_id` with cache-first pattern. Run `cargo check`.
  **Requirements**: REQ-2, REQ-4

- [x] 9. Integrate cache into attendance repository
  Add `row_to_hash_map`. Modify `find_all` and `find_by_id` with cache-first pattern. Run `cargo check`.
  **Requirements**: REQ-2, REQ-4

- [x] 10. Integrate cache into accounting repository
  Add `row_to_hash_map`. Modify `list` and `get_by_id` with cache-first pattern. Run `cargo check`.
  **Requirements**: REQ-2, REQ-4

- [x] 11. Integrate cache into user repository
  Add `row_to_hash_map`. Modify `find_all` and `find_by_id` with cache-first pattern. Run `cargo check`.
  **Requirements**: REQ-2, REQ-4

- [x] 12. Add unit tests for MemoryBuffer cache methods
  Tests: set_and_get_cached_list, set_and_get_cached_entity, cache_miss_returns_none, invalidate_table_clears_list_and_entities, invalidate_preserves_other_tables, buffer_write_invalidates_cache, non_cacheable_table_bypasses_cache, clear_user_cache. Run `cargo test`.
  **Requirements**: All (verification)

- [x] 13. Final verification
  Run `cargo check`, `cargo test`, `bunx tsc --noEmit`. Manual test: open app, navigate pages, verify reads are instant after first load.
  **Requirements**: All (end-to-end validation)

## Task Dependency Graph

```
Task 1 → Task 2 → Task 3 → Task 4
                     ↓
         ┌───┬───┬──┴──┬───┬───┬───┐
         ↓   ↓   ↓     ↓   ↓   ↓   ↓
        T5  T6  T7   T8  T9  T10 T11
         └───┴───┴──┬──┴───┴───┴───┘
                     ↓
                   Task 12 → Task 13
```

- Tasks 1-4: Sequential (each builds on previous)
- Tasks 5-11: Parallel (independent repos, all depend on Task 4)
- Task 12: Depends on Tasks 1-4 (tests core cache)
- Task 13: Depends on all tasks

## Notes

- All repository changes follow the same pattern — implement Task 5 first as reference, then replicate for Tasks 6-11
- The `row_to_hash_map` helper must match the exact column order in each repo's SELECT queries
- Cache invalidation happens synchronously inside `buffer_write` — no async needed
- No frontend changes required — backend-only optimization
- The existing `clear_cache` method in flush_timer.rs must be renamed to `clear_user_cache` to compile after Task 1
