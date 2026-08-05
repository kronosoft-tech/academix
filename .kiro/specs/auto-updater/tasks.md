# Implementation Plan: Auto-Updater

## Overview

Implement a seamless auto-update system using `tauri-plugin-updater`. The backend (Rust) handles periodic update checking via a background scheduler and exposes IPC commands. The frontend (React + Zustand) manages update state and drives the download/install flow via the `@tauri-apps/plugin-updater` JavaScript API. All core validation logic (SemVer comparison, platform matching, URL validation, etc.) is implemented in TypeScript utility functions with property-based tests to ensure correctness across large input spaces.

## Tasks

- [x] 1. Configure Tauri updater plugin and project dependencies
  - [x] 1.1 Add `tauri-plugin-updater` to Rust dependencies and register the plugin
    - Add `tauri-plugin-updater = "2"` to `src-tauri/Cargo.toml` under `[dependencies]`
    - Register `.plugin(tauri_plugin_updater::Builder::new().build())` in `src-tauri/src/lib.rs` builder chain
    - Add `"updater:default"` and `"process:default"` permissions to `src-tauri/capabilities/default.json`
    - _Requirements: 6.1, 7.1, 7.2_

  - [x] 1.2 Configure updater in `tauri.conf.json`
    - Add `plugins.updater` block with `pubkey` (placeholder), `endpoints` array with GitHub Releases URL pattern using `{{target}}` and `{{arch}}` substitution
    - Add `windows.installMode: "passive"` for Windows passive installation
    - Add `bundle.createUpdaterArtifacts: true`
    - _Requirements: 6.1, 7.1, 7.7, 4.9_

  - [x] 1.3 Install frontend dependencies
    - Run `bun add @tauri-apps/plugin-updater @tauri-apps/plugin-process`
    - Run `bun add -d fast-check` for property-based testing
    - _Requirements: 4.1, 4.2_

- [x] 2. Implement core validation utilities and types
  - [x] 2.1 Create TypeScript types and interfaces
    - Create `src/features/updater/types/updater.ts` with `UpdateInfo`, `UpdaterState`, `UpdaterActions`, and `DownloadProgress` interfaces
    - _Requirements: 3.1, 4.2_

  - [x] 2.2 Implement validation utility functions
    - Create `src/features/updater/utils/validators.ts` with:
      - `compareSemVer(current: string, remote: string): boolean` — returns true when remote > current
      - `parseUpdateManifest(payload: unknown): UpdateInfo | Error` — validates required fields
      - `constructPlatformKey(target: string, arch: string): string` — produces `{target}-{arch}`
      - `matchPlatform(manifest: object, platformKey: string): ArtifactEntry | null`
      - `validateUrl(url: string): boolean` — non-empty and ≤ 2048 chars, HTTPS-only
      - `truncateReleaseNotes(notes: string, max?: number): string` — truncates at 5000 chars
      - `validateCheckInterval(hours: number): boolean` — accepts only [1, 24]
      - `validatePublicKey(key: string): boolean` — basic Ed25519 key format validation
    - _Requirements: 1.3, 1.5, 2.1, 2.3, 2.5, 3.1, 7.2, 7.6, 6.1_

  - [x] 2.3 Write property tests for SemVer comparison
    - **Property 1: SemVer Comparison Correctness**
    - **Validates: Requirements 1.3**

  - [x] 2.4 Write property tests for manifest parsing
    - **Property 2: Invalid Payload Rejection**
    - **Validates: Requirements 1.5, 7.6**

  - [x] 2.5 Write property tests for platform key construction
    - **Property 3: Platform Key Construction**
    - **Validates: Requirements 2.1, 2.3, 7.7**

  - [x] 2.6 Write property tests for platform matching
    - **Property 4: Platform Matching Exclusion**
    - **Validates: Requirements 2.2**

  - [x] 2.7 Write property tests for URL validation
    - **Property 5: URL Validation Constraint**
    - **Validates: Requirements 2.5**

  - [x] 2.8 Write property tests for release notes truncation
    - **Property 6: Release Notes Truncation**
    - **Validates: Requirements 3.1**

  - [x] 2.9 Write property tests for check interval validation
    - **Property 8: Check Interval Persistence Round-Trip**
    - **Validates: Requirements 1.2**

  - [x] 2.10 Write property tests for HTTPS-only enforcement
    - **Property 9: HTTPS-Only Endpoint Enforcement**
    - **Validates: Requirements 7.2**

  - [x] 2.11 Write property tests for public key validation
    - **Property 11: Public Key Validation**
    - **Validates: Requirements 6.1**

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Implement Rust backend: commands and scheduler
  - [x] 4.1 Create Tauri IPC commands for the updater
    - Create `src-tauri/src/commands/updater.rs` with:
      - `check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String>` — invokes the plugin check and returns parsed info
      - `get_update_check_interval() -> Result<u64, String>` — reads persisted interval
      - `set_update_check_interval(hours: u64) -> Result<(), String>` — validates [1,24] and persists
    - Register commands in `src-tauri/src/commands/mod.rs`
    - Register commands in `lib.rs` `generate_handler![]`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

  - [x] 4.2 Implement update scheduler infrastructure
    - Create `src-tauri/src/infrastructure/updater.rs` with `UpdateScheduler` struct
    - Implement `start(app_handle: AppHandle, interval_hours: u64)` that:
      - Performs initial check 10 seconds after window ready
      - Repeats at configured interval (default 4 hours)
      - Emits `update-available` event to frontend via `AppHandle::emit`
      - Logs failures silently without disrupting app
    - Implement interval persistence to a local config file (e.g., JSON in app data dir)
    - Register the module in `src-tauri/src/infrastructure/mod.rs`
    - _Requirements: 1.1, 1.2, 1.4, 1.6_

  - [x] 4.3 Wire the scheduler into application startup
    - In `lib.rs` `run()`, spawn the scheduler in a `setup()` hook after plugin registration
    - Pass the `AppHandle` to the scheduler for event emission
    - _Requirements: 1.1_

- [x] 5. Implement frontend: Zustand store and event handling
  - [x] 5.1 Create the Zustand updater store
    - Create `src/features/updater/hooks/useUpdaterStore.ts`
    - Implement state: `status`, `updateInfo`, `downloadProgress`, `error`, `dismissedVersion`
    - Implement actions: `setStatus`, `setUpdateInfo`, `setProgress`, `setError`, `dismiss`, `reset`
    - Persist `dismissedVersion` to localStorage
    - _Requirements: 3.3, 3.4, 3.6_

  - [x] 5.2 Write property test for dismissed version suppression
    - **Property 7: Dismissed Version Suppression**
    - **Validates: Requirements 3.4**

  - [x] 5.3 Create the `useUpdater` hook
    - Create `src/features/updater/hooks/useUpdater.ts`
    - Subscribe to `update-available` Tauri event on mount (via `listen` from `@tauri-apps/api/event`)
    - Implement `checkForUpdate()` — invokes Rust command and updates store
    - Implement `startDownload()` — calls `@tauri-apps/plugin-updater` JS API `downloadAndInstall()` with progress callback
    - Implement retry logic with exponential backoff (1s, 2s, 4s) for up to 3 failures
    - Handle timeout (300s), signature failure, and installation errors
    - Trigger automatic app restart on successful install via `@tauri-apps/plugin-process` relaunch
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 5.1, 5.3, 7.5_

  - [x] 5.4 Write property test for retry backoff timing
    - **Property 10: Retry with Exponential Backoff**
    - **Validates: Requirements 7.5**

- [x] 6. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Implement frontend: UI components
  - [x] 7.1 Create the `DownloadProgress` component
    - Create `src/features/updater/components/DownloadProgress.tsx`
    - Display progress bar with percentage
    - Update at least every 2 seconds
    - _Requirements: 4.2_

  - [x] 7.2 Create the `UpdateNotification` component
    - Create `src/features/updater/components/UpdateNotification.tsx`
    - Render as a toast/banner at top of application layout
    - Display version number (MAJOR.MINOR.PATCH) and truncated release notes
    - "Download" button triggers `startDownload()`, disabled during download
    - "Dismiss" button hides notification and records dismissed version
    - Show error state with retry option
    - Only render when store `status` is `'available'`, `'downloading'`, or `'error'`
    - Display "Release notes unavailable" when notes are empty
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 4.1, 4.3_

  - [x] 7.3 Create feature barrel export and integrate into app layout
    - Create `src/features/updater/index.ts` exporting public API
    - Mount `UpdateNotification` in the main application layout
    - Initialize `useUpdater` hook at app root level
    - _Requirements: 3.3, 3.6_

- [x] 8. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Write unit tests for UI components and store
  - [x] 9.1 Write unit tests for UpdateNotification component
    - Test renders when update available
    - Test does not render when idle
    - Test dismiss hides notification
    - Test download button triggers state change
    - Test error state shows retry
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

  - [x] 9.2 Write unit tests for Zustand store transitions
    - Test state machine: idle → checking → available → downloading → installing
    - Test dismissed version persists and suppresses re-display
    - Test reset clears all state
    - _Requirements: 3.4, 4.1, 4.2_

- [x] 10. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document (Properties 1–11)
- Unit tests validate specific examples, edge cases, and UI rendering
- The `@tauri-apps/plugin-updater` JS API handles signature verification and platform-specific installation internally — Rust commands orchestrate checking; the frontend drives download/install
- The `TAURI_SIGNING_PRIVATE_KEY` setup (Requirement 6.3–6.5) is a CI/CD concern handled outside the codebase — not a coding task

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2", "1.3"] },
    { "id": 1, "tasks": ["2.1"] },
    { "id": 2, "tasks": ["2.2", "4.1", "4.2"] },
    { "id": 3, "tasks": ["2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "2.9", "2.10", "2.11", "4.3"] },
    { "id": 4, "tasks": ["5.1"] },
    { "id": 5, "tasks": ["5.2", "5.3"] },
    { "id": 6, "tasks": ["5.4", "7.1"] },
    { "id": 7, "tasks": ["7.2"] },
    { "id": 8, "tasks": ["7.3"] },
    { "id": 9, "tasks": ["9.1", "9.2"] }
  ]
}
```
