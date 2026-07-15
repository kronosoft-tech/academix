# Spec: Student Attendance Warnings

> Delta spec for change `student-assistant-warnings`
> Derived from proposal: `openspec/changes/student-assistant-warnings/proposal.md`
> Date: 2026-07-14

---

## 1. Functional Requirements

### FR-1: Absence Counting

- Count absences per student per group from the `attendance` table where `status = 'absent'`.
- Only count records where the student is enrolled in the group (`group_students.status = 'active'`).
- Absence count is computed on-demand via SQL query; no denormalized counter.
- Edge case: student with no attendance records → count = 0.
- Edge case: student enrolled in multiple groups → count is per-group, not global.

### FR-2: Warning Badge Display

- Show a warning badge when a student's absence count **exceeds** (strictly greater than) the configurable threshold.
- Badge appears in three locations:
  1. **StudentsPage table**: New "Asistencia" column in the student list table.
  2. **StudentDetailModal**: "Asistencia" section showing per-group absence counts.
  3. **GroupDetailPage (AtRiskStudentsWidget)**: List of at-risk students in the group attendance view.
- All warning text in Spanish: "Más de X faltas", "Advertencia", "Estudiantes en riesgo", "No hay estudiantes en riesgo".

### FR-3: Configurable Threshold

- Default threshold: **3** (students with >3 absences trigger warnings).
- Users can change the threshold via an `AttendanceThresholdSettings` component.
- Minimum allowed value: **1**.
- Maximum allowed value: **30**.
- Threshold persists across sessions (stored in `app_settings` table, not localStorage — see Data Model).

### FR-4: At-Risk Students Widget

- Displayed in the `GroupDetailPage` (not `DailyAttendanceForm` — see affected areas).
- Lists all students in the group whose absence count exceeds the threshold.
- Shows: student name, absence count, severity indicator (badge variant).
- Updates in real-time after attendance is saved (the parent `GroupDetailPage` re-fetches on attendance save).
- Empty state: "No hay estudiantes en riesgo".

---

## 2. Data Model

### 2.1 Absence Count Computation (SQL)

No new tables needed. Absence count is computed via:

```sql
-- Single student, single group
SELECT COUNT(*) as absence_count
FROM attendance
WHERE student_id = ? AND group_id = ? AND status = 'absent';

-- All students in a group (batch)
SELECT student_id, COUNT(*) as absence_count
FROM attendance
WHERE group_id = ? AND status = 'absent'
GROUP BY student_id;
```

**Existing indexes** (`idx_attendance_student_id`, `idx_attendance_group_id`) are sufficient for these queries. No new indexes needed.

### 2.2 Threshold Storage

New migration: `017_add_app_settings.sql`

```sql
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings (key, value, updated_at)
VALUES ('attendance_threshold', '3', datetime('now'));
```

**Rationale**: Using a database table (not localStorage) keeps settings consistent if the app is used on multiple machines with the same database, and follows the project's existing backend pattern.

### 2.3 New Types (Frontend)

```typescript
// src/shared/types/AttendanceWarning.ts
export interface StudentAbsenceCount {
  studentId: string;
  absenceCount: number;
  isAtRisk: boolean; // absenceCount > threshold
}

export interface AbsenceThreshold {
  value: number; // default: 3, min: 1, max: 30
}
```

---

## 3. API Contract

### 3.1 `get_student_absence_count`

```rust
#[tauri::command]
pub fn get_student_absence_count(
    state: State<AttendanceServiceState>,
    student_id: String,
    group_id: String,
) -> AbsenceCountResponse
```

**Request**: `{ student_id: string, group_id: string }`

**Response**:
```rust
struct AbsenceCountResponse {
    success: bool,
    data: Option<AbsenceCountDto>,
    error: Option<String>,
}

struct AbsenceCountDto {
    student_id: String,
    group_id: String,
    absence_count: i32,
}
```

### 3.2 `get_students_absence_counts` (Batch)

```rust
#[tauri::command]
pub fn get_students_absence_counts(
    state: State<AttendanceServiceState>,
    group_id: String,
) -> AbsenceCountListResponse
```

**Request**: `{ group_id: string }`

**Response**:
```rust
struct AbsenceCountListResponse {
    success: bool,
    data: Option<Vec<AbsenceCountDto>>,
    error: Option<String>,
}
```

Returns all students in the group with their absence counts (0 for students with no absences). This enables batch loading in the StudentsPage and AtRiskStudentsWidget.

### 3.3 `get_at_risk_students`

```rust
#[tauri::command]
pub fn get_at_risk_students(
    state: State<AttendanceServiceState>,
    group_id: String,
    threshold: i32,
) -> AtRiskStudentsResponse
```

**Request**: `{ group_id: string, threshold: i32 }`

**Response**:
```rust
struct AtRiskStudentsResponse {
    success: bool,
    data: Option<Vec<AtRiskStudentDto>>,
    error: Option<String>,
}

struct AtRiskStudentDto {
    student_id: String,
    student_name: String,
    absence_count: i32,
}
```

Returns only students with `absence_count > threshold`, sorted by `absence_count DESC`.

### 3.4 `get_absence_threshold`

```rust
#[tauri::command]
pub fn get_absence_threshold(
    state: State<SettingsServiceState>,
) -> ThresholdResponse
```

**Response**:
```rust
struct ThresholdResponse {
    success: bool,
    data: Option<ThresholdDto>,
    error: Option<String>,
}

struct ThresholdDto {
    value: i32,
}
```

### 3.5 `set_absence_threshold`

```rust
#[tauri::command]
pub fn set_absence_threshold(
    state: State<SettingsServiceState>,
    value: i32,
) -> ThresholdResponse
```

**Request**: `{ value: i32 }`

Validates `1 <= value <= 30`. Returns the updated threshold on success.

---

## 4. UI Specifications

### 4.1 `AttendanceWarningBadge` Component

**File**: `src/features/students/components/AttendanceWarningBadge.tsx`

**Props**:
```typescript
interface AttendanceWarningBadgeProps {
  absenceCount: number;
  threshold: number;
  showCount?: boolean; // default: true — show "X faltas" text
}
```

**Behavior**:
- If `absenceCount <= threshold`: render nothing (null).
- If `absenceCount > threshold`: render a `<Badge variant="danger">` with text `"Más de {threshold} faltas"`.
- When `showCount = false`: render `<Badge variant="danger">Advertencia</Badge>`.

**Styling**: Reuses existing `Badge` component from `src/shared/ui/components/Badge.tsx` with `danger` variant.

### 4.2 StudentsPage Integration

**File**: `src/features/students/routes/StudentsPage.tsx`

**Changes**:
1. Add new table header column "Asistencia" between "Teléfono" and "Estado" columns.
2. For each student row, fetch absence counts via the batch command (single call for all students).
3. Render `<AttendanceWarningBadge>` in the new column.
4. Pass `threshold` from `get_absence_threshold()` (fetched once on mount).

**Loading state**: Show spinner in the "Asistencia" column while absence counts load. The existing payment loading pattern can be reused.

### 4.3 StudentDetailModal Integration

**File**: `src/features/students/routes/StudentsPage.tsx` (the modal is inline in this file)

**Changes**: Add an "Asistencia" section inside the existing details modal, after the "Inscripción" section:

```
┌─ Asistencia ──────────────────────────┐
│ Grupo: [GroupName] — 5 faltas ⚠️     │
│ Grupo: [GroupName2] — 1 faltas        │
└────────────────────────────────────────┘
```

Each group entry shows the group name and absence count with the badge if at threshold.

### 4.4 `AtRiskStudentsWidget` Component

**File**: `src/features/groups/components/AtRiskStudentsWidget.tsx`

**Props**:
```typescript
interface AtRiskStudentsWidgetProps {
  groupId: string;
  threshold: number;
}
```

**Behavior**:
1. Fetches at-risk students via `get_at_risk_students(groupId, threshold)`.
2. Displays a card with header "Estudiantes en riesgo" and a list of students.
3. Each row shows: student name (left), absence count badge (right).
4. Empty state: "No hay estudiantes en riesgo" with muted styling.
5. Refreshes when `groupId` prop changes.

**Placement**: Rendered inside `GroupDetailPage.tsx`, above the `DailyAttendanceForm`. This ensures it's visible when the teacher opens a group's attendance page.

### 4.5 `AttendanceThresholdSettings` Component

**File**: `src/features/students/components/AttendanceThresholdSettings.tsx`

**Props**: None (self-contained).

**Behavior**:
1. On mount, fetches current threshold via `get_absence_threshold()`.
2. Renders a number input with label "Umbral de advertencia de asistencia".
3. Helper text: "Los estudiantes con más de esta cantidad de faltas serán marcados con advertencia".
4. Save button triggers `set_absence_threshold(value)`.
5. Validates min=1, max=30 on the input.
6. Shows success/error toast after save.

**Placement**: Rendered in `StudentsPage` as a collapsible settings section at the top, or as a gear icon that opens a small popover. Decision deferred to implementation — spec requires it to be accessible from the StudentsPage.

---

## 5. Affected Areas

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/migrations/017_add_app_settings.sql` | Created | New `app_settings` table |
| `src-tauri/src/lib.rs` | Modified | Register new commands, add migration, create SettingsService |
| `src-tauri/src/commands/attendance.rs` | Modified | Add 3 new commands: absence count, batch count, at-risk |
| `src-tauri/src/commands/settings.rs` | Created | Threshold get/set commands |
| `src-tauri/src/application/use_cases/attendance.rs` | Modified | Add `count_absences`, `count_absences_batch`, `get_at_risk_students` methods |
| `src-tauri/src/application/use_cases/settings.rs` | Created | Settings service for threshold management |
| `src-tauri/src/application/ports/settings.rs` | Created | Settings repository port |
| `src-tauri/src/infrastructure/repositories/sqlite/settings.rs` | Created | SQLite settings repository |
| `src/shared/types/AttendanceWarning.ts` | Created | Frontend types |
| `src/features/students/hooks/useStudentAbsences.ts` | Created | Hook wrapping absence commands |
| `src/features/students/components/AttendanceWarningBadge.tsx` | Created | Warning badge component |
| `src/features/students/components/AttendanceThresholdSettings.tsx` | Created | Threshold settings component |
| `src/features/students/routes/StudentsPage.tsx` | Modified | Add absence column, badge, settings, modal section |
| `src/features/groups/components/AtRiskStudentsWidget.tsx` | Created | At-risk students widget |
| `src/features/groups/components/GroupDetailInline.tsx` | Modified | Integrate AtRiskStudentsWidget |
| `src/features/groups/hooks/useAttendance.ts` | Modified | Add `getAtRiskStudents` function |

---

## 6. Acceptance Criteria

### Backend
- [ ] **AC-B1**: `get_student_absence_count` returns correct count for a student in a group.
- [ ] **AC-B2**: `get_student_absence_count` returns 0 when student has no attendance records.
- [ ] **AC-B3**: `get_students_absence_counts` returns counts for all students in a group (including those with 0 absences).
- [ ] **AC-B4**: `get_at_risk_students` returns only students with `absence_count > threshold`.
- [ ] **AC-B5**: `get_at_risk_students` returns students sorted by `absence_count DESC`.
- [ ] **AC-B6**: `get_absence_threshold` returns the stored threshold (default 3).
- [ ] **AC-B7**: `set_absence_threshold` persists value and rejects values outside [1, 30].
- [ ] **AC-B8**: New migration `017_add_app_settings.sql` creates `app_settings` table with default threshold.

### Frontend
- [ ] **AC-F1**: `AttendanceWarningBadge` renders nothing when `absenceCount <= threshold`.
- [ ] **AC-F2**: `AttendanceWarningBadge` renders red badge with "Más de X faltas" when `absenceCount > threshold`.
- [ ] **AC-F3**: StudentsPage shows "Asistencia" column with badge for each student.
- [ ] **AC-F4**: StudentDetailModal shows per-group absence counts in "Asistencia" section.
- [ ] **AC-F5**: `AtRiskStudentsWidget` lists students exceeding threshold, sorted by absence count.
- [ ] **AC-F6**: `AtRiskStudentsWidget` shows "No hay estudiantes en riesgo" when list is empty.
- [ ] **AC-F7**: `AttendanceThresholdSettings` saves threshold and validates range [1, 30].
- [ ] **AC-F8**: All warning text is in Spanish.
- [ ] **AC-F9**: No performance degradation: StudentsPage loads absence data in a single batch call.
- [ ] **AC-F10**: `AttendanceWarningBadge` reuses existing `Badge` component variants.

---

## 7. Non-Functional Requirements

- **NFR-1**: Absence count queries execute in < 50ms for groups with up to 100 students and 1000 attendance records.
- **NFR-2**: Batch absence count loads all students in one Tauri command invocation (no N+1).
- **NFR-3**: Settings changes take effect immediately without page reload.
- **NFR-4**: No database schema changes to existing tables (only additive `app_settings` table).

---

## 8. Open Questions

| # | Question | Resolution |
|---|----------|------------|
| 1 | Should "late" and "excused" statuses count as absences? | **No** — only `status = 'absent'` counts. This matches the proposal's SQL query specification. |
| 2 | Where exactly does AttendanceThresholdSettings render in StudentsPage? | Deferred to implementation. Spec requires it to be accessible from StudentsPage. |
| 3 | Should the AtRiskStudentsWidget auto-refresh after attendance is saved? | Yes — the parent `GroupDetailPage` should re-fetch on `onRefresh` callback from `DailyAttendanceForm`. |
