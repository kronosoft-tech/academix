# Offline Sync: Embedded Replica

> New specification for offline-first capability using Turso Embedded Replicas.

## Purpose

The system SHALL allow users to work without internet connectivity by maintaining a local SQLite replica of their Turso database. When connectivity is restored, the system SHALL automatically synchronize local changes to the cloud.

---

## ADDED Requirements

### Requirement: Local Embedded Replica

On startup (or first login after migration), the system SHALL create a local embedded replica of the user's Turso database using `libsql::Database::open_with_remote_sync()`. The local replica file SHALL be stored at `{app_data_dir}/academix/replicas/{user_id}.db`.

All database reads and writes SHALL happen against the local replica for zero-latency access.

#### Scenario: First login creates local replica

- GIVEN user A logs in for the first time after Turso migration
- WHEN the connection to user A's Turso DB is established
- THEN a local replica file is created at `{app_data}/replicas/{user_id}.db`
- AND the replica is synced with the remote Turso DB on creation

#### Scenario: Subsequent logins use existing replica

- GIVEN user A has an existing local replica at `{app_data}/replicas/{user_id}.db`
- WHEN user A logs in again
- THEN the existing local replica is reused
- AND it's synced with the remote on startup

### Requirement: Automatic Sync on Connectivity

The system SHALL attempt to synchronize the local replica with the remote Turso database in the following scenarios:
1. On application startup (sync pull before use)
2. After reconnecting from network loss (sync push + pull)
3. Every 5 minutes while connected (sync push)
4. Before performing sensitive operations (sync push — optional, configurable)

Sync SHALL use `Connection::sync()` which performs a bidirectional synchronization.

#### Scenario: Sync on startup

- GIVEN the app starts with internet connectivity
- WHEN the user's database connection is initialized
- THEN `sync()` is called to pull the latest remote changes
- AND the local replica is up-to-date with the cloud

#### Scenario: Sync on reconnect

- GIVEN the user works offline and creates 3 new student records
- WHEN internet connectivity is restored
- THEN the system automatically calls `sync()`
- AND the 3 new records are pushed to the Turso cloud database
- AND any remote changes are pulled to the local replica

### Requirement: Offline Resilience

When there is no internet connectivity, the system SHALL:
1. Continue reading from and writing to the local replica (WAL mode)
2. Queue changes locally (they accumulate in the WAL)
3. NOT show errors for data operations
4. Show a subtle offline indicator in the UI (optional, non-blocking)

#### Scenario: Read works offline

- GIVEN the user has no internet connection
- WHEN the user opens the Students page
- THEN the last synced student data is displayed
- AND no connection error is shown

#### Scenario: Write works offline

- GIVEN the user has no internet connection
- WHEN the user creates a new payment record
- THEN the record is saved to the local replica
- AND the operation succeeds without error
- AND the data is queued for sync when connectivity returns

### Requirement: Sync Conflict Resolution

For a single-user-per-DB architecture, conflicts are rare. The system SHALL use Turso's default Last-Write-Wins (LWW) strategy for conflict resolution.

#### Scenario: Local change after disconnect syncs correctly

- GIVEN the user creates a new student "Alice" while offline
- WHEN connectivity is restored and sync runs
- THEN the "Alice" record is pushed to Turso
- AND no conflict occurs (new records don't conflict)

### Requirement: Local Replica Cleanup

When a user is deleted or the app data is cleared, the local replica files SHALL be removed. If a local replica becomes corrupted, the system SHALL detect this on startup, delete it, and recreate a fresh replica from the remote.

#### Scenario: Corrupted replica is rebuilt

- GIVEN the local replica file is corrupted
- WHEN the app starts and tries to sync
- THEN the sync fails
- AND the system deletes the corrupted replica
- AND creates a fresh replica synced from Turso
