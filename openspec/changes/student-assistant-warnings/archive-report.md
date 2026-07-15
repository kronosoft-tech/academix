# Archive Report: student-assistant-warnings

> Archived: 2026-07-14
> Status: ✅ COMPLETE

---

## Summary

Added a warning system that alerts when students exceed a configurable absence threshold (default: 3). Warnings appear in the student list, student detail modal, and group attendance page.

## What Was Built

### Backend (Rust)
- Migration `017_add_app_settings.sql` — new settings table
- `SettingsRepository` port + SQLite implementation
- `SettingsService` use case for threshold management
- `count_student_absences` command
- `count_group_absences` batch command
- `get_absence_threshold` / `set_absence_threshold` commands

### Frontend (React/TypeScript)
- `AttendanceWarning.ts` types
- `useStudentAbsences` hook
- `useAttendanceThreshold` hook
- `AttendanceWarningBadge` component
- `AttendanceThresholdSettings` component
- `AtRiskStudentsWidget` component
- StudentsPage: added "Asistencia" column + threshold settings
- StudentDetailModal: added per-group absence section
- GroupDetailInline: integrated AtRiskStudentsWidget

## Files Changed

| File | Action |
|------|--------|
| `src-tauri/migrations/017_add_app_settings.sql` | Created |
| `src-tauri/src/commands/settings.rs` | Created |
| `src-tauri/src/application/ports/settings.rs` | Created |
| `src-tauri/src/application/use_cases/settings.rs` | Created |
| `src-tauri/src/infrastructure/repositories/sqlite/settings.rs` | Created |
| `src-tauri/src/commands/attendance.rs` | Modified |
| `src-tauri/src/application/ports/attendance.rs` | Modified |
| `src-tauri/src/application/use_cases/attendance.rs` | Modified |
| `src-tauri/src/infrastructure/repositories/sqlite/attendance.rs` | Modified |
| `src-tauri/src/application/dto/attendance.rs` | Modified |
| `src-tauri/src/lib.rs` | Modified |
| `src/shared/types/AttendanceWarning.ts` | Created |
| `src/features/students/hooks/useStudentAbsences.ts` | Created |
| `src/features/students/hooks/useAttendanceThreshold.ts` | Created |
| `src/features/students/components/AttendanceWarningBadge.tsx` | Created |
| `src/features/students/components/AttendanceThresholdSettings.tsx` | Created |
| `src/features/groups/components/AtRiskStudentsWidget.tsx` | Created |
| `src/features/students/routes/StudentsPage.tsx` | Modified |
| `src/features/groups/components/GroupDetailInline.tsx` | Modified |

## Verification

- ✅ `cargo check` — Rust compiles
- ✅ `bunx tsc --noEmit` — TypeScript compiles
- ✅ `bun run build` — Production build succeeds

## Deviations from Spec

1. **AtRiskStudentsWidget uses batch query** — Instead of separate `get_at_risk_students` command, widget calls `count_group_absences` and filters client-side. Functionally equivalent, avoids N+1.
2. **Badge text** — Uses "Más de {threshold} faltas" as specified.

## Follow-up Items

- None — feature is complete and verified.
