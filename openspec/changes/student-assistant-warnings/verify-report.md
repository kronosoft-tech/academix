# Verify Report: Student Assistant Warnings

> Change: `student-assistant-warnings`
> Date: 2026-07-14
> Verifier: AI Agent (big-pickle)

---

## Overall Status: PARTIAL

---

## 1. File Verification

| File | Status | Notes |
|------|--------|-------|
| `src-tauri/migrations/017_add_app_settings.sql` | ✅ | Created, correct schema + default threshold |
| `src-tauri/src/application/ports/settings.rs` | ✅ | Created, correct trait |
| `src-tauri/src/application/ports/mod.rs` | ✅ | Contains `pub mod settings;` |
| `src-tauri/src/application/use_cases/settings.rs` | ✅ | Created with threshold validation (1..=30) |
| `src-tauri/src/application/use_cases/mod.rs` | ✅ | Contains `pub mod settings;` |
| `src-tauri/src/infrastructure/repositories/sqlite/settings.rs` | ✅ | Created, correct SQL queries |
| `src-tauri/src/infrastructure/repositories/sqlite/mod.rs` | ✅ | Exports `SqliteSettingsRepository` |
| `src-tauri/src/commands/settings.rs` | ✅ | Created with `get_absence_threshold`, `set_absence_threshold` |
| `src-tauri/src/commands/mod.rs` | ✅ | Contains `pub mod settings;` |
| `src-tauri/src/lib.rs` | ✅ | Migration 017, settings service, commands registered |
| `src/shared/types/AttendanceWarning.ts` | ✅ | Created, exports interfaces |
| `src/shared/types/index.ts` | ✅ | Re-exports all types |
| `src/features/students/hooks/useStudentAbsences.ts` | ✅ | Created |
| `src/features/students/hooks/useAttendanceThreshold.ts` | ✅ | Created |
| `src/features/students/components/AttendanceWarningBadge.tsx` | ✅ | Created |
| `src/features/students/components/AttendanceThresholdSettings.tsx` | ✅ | Created |
| `src/features/groups/components/AtRiskStudentsWidget.tsx` | ✅ | Created |

## 2. Build Verification

| Check | Status | Details |
|-------|--------|---------|
| `cargo check` | ✅ | Compilation successful |
| `bunx tsc --noEmit` | ✅ | No type errors |
| `bun run build` | ✅ | Vite production build successful |

## 3. Spec Compliance

### Backend (Rust)

| Criteria | Status | Notes |
|----------|--------|-------|
| AC-B1: `count_student_absences` returns correct count | ✅ | SQL COUNT with status='absent' filter |
| AC-B2: Returns 0 for no records | ✅ | SQL COUNT returns 0 naturally |
| AC-B3: Batch returns all students | ✅ | COUNT + GROUP BY |
| AC-B4: `get_at_risk_students` filters > threshold | ❌ | Missing — no separate `get_at_risk_students` command; filtering done on client |
| AC-B5: Sorted by count DESC | ❌ | No server-side sorting; sorting done on client |
| AC-B6: `get_absence_threshold` returns default 3 | ✅ | Returns 3 if no row in DB |
| AC-B7: `set_absence_threshold` validates [1, 30] | ✅ | Returns error for out-of-range |
| AC-B8: Migration creates table with default | ✅ | Correct SQL in migration file |
| NFR-4: No existing table changes | ✅ | Only new `app_settings` table |

### Frontend

| Criteria | Status | Notes |
|----------|--------|-------|
| AC-F1: Badge hides when count <= threshold | ✅ | Returns null |
| AC-F2: Badge shows when count > threshold | ✅ | Shows `"{N} faltas"` — **text differs from spec** (spec says `"Más de X faltas"`) |
| AC-F3: StudentsPage "Asistencia" column | ✅ | Column between Teléfono and Estado |
| AC-F4: StudentDetailModal shows per-group counts | ✅ | "Asistencia" section after Inscripción |
| AC-F5: AtRiskStudentsWidget lists students > threshold | ✅ | Done client-side |
| AC-F6: Empty state "No hay estudiantes en riesgo" | ✅ | Correct empty state text |
| AC-F7: Threshold settings validates range | ✅ | Client validates min/max |
| AC-F8: All text in Spanish | ✅ | All UI text in Spanish |
| AC-F10: Badge reuses existing Badge component | ✅ | Imports from shared/ui |
| Student names in widget | ❌ | `student_name: s.student_id` — shows IDs, not actual student names |
| At-risk: auto-refresh on attendance save | ⚠️ | `onRefresh` callback is empty `() => {}` in GroupDetailInline |

## 4. Issues Found

### Critical
1. **Missing `get_at_risk_students` command**: The spec defines a dedicated `get_at_risk_students` Tauri command (with threshold filtering + sorting on server). The implementation uses `count_group_absences` and does filtering on the client side. The `AtRiskStudentDto` struct exists but is unused in any command.

2. **Student names not resolved in AtRiskStudentsWidget**: The widget assigns `student_name: s.student_id` because the `count_group_absences` query only returns `student_id` and `absence_count`. Student names are never fetched. The spec requires names to be displayed.

### Minor
3. **Badge text differs from spec**: Spec says `"Más de {threshold} faltas"`, implementation shows `"{absenceCount} faltas"` (shows actual count, not threshold).

4. **onRefresh empty**: `GroupDetailInline` passes `onRefresh={() => {}}` to `DailyAttendanceForm`. The widget does not auto-refresh after attendance save per spec requirement.

### Note
- Commands `count_student_absences` and `count_group_absences` are registered instead of the spec's named commands. The frontend code uses these API names correctly.

## 5. Task Completion

| Phase | Tasks | Done | Notes |
|-------|-------|------|-------|
| Phase 1 (Backend) | 1.1-1.11 | 11/11 | All checked |
| Phase 2 (Backend absence counting) | 2.1-2.7 | 7/7 | All checked |
| Phase 3 (Frontend types & hooks) | 3.1-3.5 | 5/5 | All checked |
| Phase 4 (UI Components) | 4.1-4.4 | 4/4 | All checked |
| Phase 5 (Integration) | 5.1-5.5 | 5/5 | All checked |
| Phase 6 (Verification) | 6.1-6.8 | 0/8 | Not started (these are test tasks) |

---

## Summary

The implementation is structurally complete — all files exist, all modules compile, both backend and frontend build cleanly. However, there are two spec-compliance issues:

1. **No `get_at_risk_students` command** → at-risk filtering is client-side, and student names are not resolved (IDs shown instead of names).
2. **Badge text** shows count instead of threshold text from spec.

These are fixable issues. The core architecture, database, and integration are solid.
