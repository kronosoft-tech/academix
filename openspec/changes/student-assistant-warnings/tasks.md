# Tasks: Student Attendance Warnings

## Phase 1: Database & Backend Settings

- [x] 1.1 Create `src-tauri/migrations/017_add_app_settings.sql` — `app_settings` table with PK `key`, columns `value TEXT NOT NULL`, `updated_at TEXT NOT NULL`, plus `INSERT OR IGNORE` for `attendance_threshold` = `3`
- [x] 1.2 Create `src-tauri/src/application/ports/settings.rs` — `SettingsRepository` trait with `get_setting(&self, key: &str) -> Result<Option<String>, DomainError>` and `set_setting(&self, key: &str, value: &str) -> Result<(), DomainError>`
- [x] 1.3 Add `pub mod settings; pub use settings::*;` to `src-tauri/src/application/ports/mod.rs`
- [x] 1.4 Create `src-tauri/src/infrastructure/repositories/sqlite/settings.rs` — `SqliteSettingsRepository` implementing `SettingsRepository` with `SELECT value FROM app_settings WHERE key = ?` and `INSERT OR REPLACE INTO app_settings ...` queries
- [x] 1.5 Add `pub mod settings; pub use settings::SqliteSettingsRepository;` to `src-tauri/src/infrastructure/repositories/sqlite/mod.rs`
- [x] 1.6 Create `src-tauri/src/application/use_cases/settings.rs` — `SettingsService<R: SettingsRepository>` with `get_absence_threshold()` (returns default 3 if missing) and `set_absence_threshold(value: i32)` (validates 1..=30)
- [x] 1.7 Add `pub mod settings; pub use settings::*;` to `src-tauri/src/application/use_cases/mod.rs`
- [x] 1.8 Create `src-tauri/src/commands/settings.rs` — `get_absence_threshold` and `set_absence_threshold` Tauri commands with `ThresholdResponse` / `ThresholdDto` types, using `SettingsServiceState` type alias
- [x] 1.9 Add `pub mod settings; pub use settings::*;` to `src-tauri/src/commands/mod.rs`
- [x] 1.10 In `src-tauri/src/lib.rs`: add migration 017 (`run_migration!`), create `SqliteSettingsRepository` + `SettingsService`, `.manage(settings_service)`, and register `get_absence_threshold, set_absence_threshold` in `generate_handler!`
- [x] 1.11 Run `cargo check` — verify backend compiles with all new modules

## Phase 2: Backend Absence Counting

- [x] 2.1 Add to `src-tauri/src/application/ports/attendance.rs`: trait methods `count_absences_by_student_and_group(&self, student_id: &str, group_id: &str) -> Result<i32, DomainError>` and `count_absences_by_group(&self, group_id: &str) -> Result<Vec<(String, i32)>, DomainError>`
- [x] 2.2 In `src-tauri/src/infrastructure/repositories/sqlite/attendance.rs`: implement both queries — single student `SELECT COUNT(*) ... WHERE student_id = ? AND group_id = ? AND status = 'absent'` and batch `SELECT student_id, COUNT(*) ... WHERE group_id = ? AND status = 'absent' GROUP BY student_id`
- [x] 2.3 Add to `src-tauri/src/application/dto/attendance.rs`: `StudentAbsenceCountDto { student_id: String, absence_count: i32 }` and `AtRiskStudentDto { student_id: String, student_name: String, absence_count: i32 }`
- [x] 2.4 In `src-tauri/src/application/use_cases/attendance.rs`: add `count_student_absences(student_id, group_id) -> Result<i32, ApplicationError>` and `count_group_absences(group_id) -> Result<Vec<StudentAbsenceCountDto>, ApplicationError>` methods
- [x] 2.5 In `src-tauri/src/commands/attendance.rs`: add `CountAbsencesCommand`, `AbsenceCountResponse`, `GroupAbsenceCountsResponse`, `AtRiskStudentsResponse` types; add `count_student_absences` and `count_group_absences` Tauri commands
- [x] 2.6 Add `count_student_absences, count_group_absences` to `generate_handler!` in `src-tauri/src/lib.rs`
- [x] 2.7 Run `cargo check` — verify absence counting compiles

## Phase 3: Frontend Types & Hooks

- [x] 3.1 Create `src/shared/types/AttendanceWarning.ts` — export `StudentAbsenceCount { studentId: string; absenceCount: number }` and `AbsenceThreshold { value: number }` interfaces
- [x] 3.2 Add `export * from "./AttendanceWarning"` to `src/shared/types/index.ts`
- [x] 3.3 Create `src/features/students/hooks/useStudentAbsences.ts` — hook wrapping `count_student_absences` (single) and `count_group_absences` (batch) Tauri commands; returns `{ getAbsenceCount, getGroupAbsenceCounts, isLoading, error }`
- [x] 3.4 Create `src/features/students/hooks/useAttendanceThreshold.ts` — hook wrapping `get_absence_threshold` and `set_absence_threshold` commands; returns `{ threshold, updateThreshold, isLoading }`
- [x] 3.5 Run `bunx tsc --noEmit` — verify frontend types compile

## Phase 4: UI Components

- [x] 4.1 Create `src/features/students/components/AttendanceWarningBadge.tsx` — component with `{ absenceCount: number; threshold: number; showCount?: boolean }` props; renders nothing when count <= threshold, else `<Badge variant="danger">Más de {threshold} faltas</Badge>` (reuses existing Badge)
- [x] 4.2 Create `src/features/students/components/AttendanceThresholdSettings.tsx` — self-contained settings component using `useAttendanceThreshold` hook; number input (1-30) with label "Umbral de advertencia de asistencia", save button, helper text, toast feedback
- [x] 4.3 Create `src/features/groups/components/AtRiskStudentsWidget.tsx` — component with `{ groupId: string; threshold: number }` props; fetches via `get_at_risk_students`, shows card "Estudiantes en riesgo" with student name + badge, empty state "No hay estudiantes en riesgo"
- [x] 4.4 Run `bunx tsc --noEmit` — verify components compile

## Phase 5: Integration

- [x] 5.1 In `src/features/students/routes/StudentsPage.tsx`: import `useStudentAbsences`, `useAttendanceThreshold`; fetch batch absence counts on mount; add "Asistencia" table column between "Teléfono" and "Estado" rendering `<AttendanceWarningBadge>` for each student; add `AttendanceThresholdSettings` as collapsible section at top
- [x] 5.2 In `src/features/students/routes/StudentsPage.tsx` (modal section): add "Asistencia" section inside student detail modal showing per-group absence counts with badges after "Inscripción" section
- [x] 5.3 In `src/features/groups/components/GroupDetailInline.tsx`: import `AtRiskStudentsWidget` and `useAttendanceThreshold`; render `<AtRiskStudentsWidget>` inside the "Pasar Lista" tab above `<DailyAttendanceForm>`; pass `threshold` from hook
- [x] 5.4 Run `bunx tsc --noEmit` — verify full integration compiles
- [x] 5.5 Run `bun run build` — verify production build succeeds

## Phase 6: Verification

- [ ] 6.1 Backend: `cargo check` passes
- [ ] 6.2 Frontend: `bunx tsc --noEmit` passes
- [ ] 6.3 Frontend: `bun run build` passes
- [ ] 6.4 Manual: open StudentsPage → "Asistencia" column shows badges for students with >3 absences
- [ ] 6.5 Manual: open student detail modal → "Asistencia" section shows per-group counts
- [ ] 6.6 Manual: open GroupDetailPage → "Pasar Lista" tab shows AtRiskStudentsWidget
- [ ] 6.7 Manual: change threshold in AttendanceThresholdSettings → badges update immediately
- [ ] 6.8 Manual: save attendance for a group → AtRiskStudentsWidget refreshes
