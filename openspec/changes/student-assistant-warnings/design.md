# Technical Design: Student Attendance Warnings

## Overview

Add absence warning badges to student list and detail views, plus an at-risk students widget in the group attendance page. The warning threshold is configurable (default: 3 absences).

## 1. Backend Architecture

### 1.1 New Repository Method

Add `count_absences_by_student_and_group` to `AttendanceRepository` trait:

```rust
// src-tauri/src/application/ports/attendance.rs
fn count_absences_by_student_and_group(
    &self,
    student_id: &str,
    group_id: &str,
) -> Result<i32, DomainError>;

fn count_absences_by_group(
    &self,
    group_id: &str,
) -> Result<Vec<(String, i32)>, DomainError>;
```

### 1.2 SQLite Repository Implementation

```sql
-- Single student absence count
SELECT COUNT(*) FROM attendance
WHERE student_id = ? AND group_id = ? AND status = 'absent'

-- All students in group with absence counts
SELECT student_id, COUNT(*) as absence_count FROM attendance
WHERE group_id = ? AND status = 'absent'
GROUP BY student_id
```

File: `src-tauri/src/infrastructure/repositories/sqlite/attendance.rs`

### 1.3 New DTOs

```rust
// src-tauri/src/application/dto/attendance.rs
#[derive(Debug, Serialize)]
pub struct StudentAbsenceCount {
    pub student_id: String,
    pub absence_count: i32,
}
```

### 1.4 New Use Case Methods

```rust
// src-tauri/src/application/use_cases/attendance.rs
pub fn count_student_absences(
    &self,
    student_id: &str,
    group_id: &str,
) -> Result<i32, ApplicationError>

pub fn count_group_absences(
    &self,
    group_id: &str,
) -> Result<Vec<StudentAbsenceCount>, ApplicationError>
```

### 1.5 New Tauri Commands

```rust
// src-tauri/src/commands/attendance.rs

#[derive(Debug, Deserialize)]
pub struct CountAbsencesCommand {
    pub student_id: String,
    pub group_id: String,
}

#[derive(Debug, Serialize)]
pub struct AbsenceCountResponse {
    pub success: bool,
    pub data: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupAbsenceCountsResponse {
    pub success: bool,
    pub data: Option<Vec<StudentAbsenceCount>>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn count_student_absences(
    state: State<AttendanceServiceState>,
    student_id: String,
    group_id: String,
) -> AbsenceCountResponse

#[tauri::command]
pub fn count_group_absences(
    state: State<AttendanceServiceState>,
    group_id: String,
) -> GroupAbsenceCountsResponse
```

### 1.6 Command Registration

Add to `src-tauri/src/lib.rs` `generate_handler!` macro:

```rust
count_student_absences,
count_group_absences,
```

## 2. Frontend Architecture

### 2.1 Types

```typescript
// src/shared/types/Attendance.ts (add)
export interface StudentAbsenceCount {
  studentId: string;
  absenceCount: number;
}

// src/features/students/types/students.ts (add)
export interface AttendanceThreshold {
  count: number;
}
```

### 2.2 useStudentAbsences Hook

File: `src/features/students/hooks/useStudentAbsences.ts`

```typescript
interface UseStudentAbsencesReturn {
  getAbsenceCount: (studentId: string, groupId: string) => Promise<number>;
  getGroupAbsenceCounts: (groupId: string) => Promise<StudentAbsenceCount[]>;
  isLoading: boolean;
  error: string | null;
}
```

Features:
- Calls `count_student_absences` for single student
- Calls `count_group_absences` for batch loading
- No caching (data fetched on demand, small payload)

### 2.3 Threshold Storage (Database)

Threshold stored in `app_settings` table, not localStorage.

**New migration**: `src-tauri/migrations/017_add_app_settings.sql`

```sql
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings (key, value, updated_at)
VALUES ('attendance_threshold', '3', datetime('now'));
```

**Rationale**: Database storage keeps settings consistent across machines sharing the same DB file, follows the project's backend pattern, and avoids localStorage limitations in Tauri WebView.

**Backend commands** (see section 1.5 additions below):

```rust
// src-tauri/src/commands/settings.rs

#[tauri::command]
pub fn get_absence_threshold(state: State<SettingsServiceState>) -> ThresholdResponse

#[tauri::command]
pub fn set_absence_threshold(state: State<SettingsServiceState>, value: i32) -> ThresholdResponse
```

**New port**: `src-tauri/src/application/ports/settings.rs`

```rust
fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError>;
fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError>;
```

**New repository**: `src-tauri/src/infrastructure/repositories/sqlite/settings.rs`

```sql
SELECT value FROM app_settings WHERE key = ?;
INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, datetime('now'));
```

**New use case**: `src-tauri/src/application/use_cases/settings.rs`

```rust
pub fn get_absence_threshold(&self) -> Result<i32, ApplicationError>;
pub fn set_absence_threshold(&self, value: i32) -> Result<i32, ApplicationError>;
```

**Frontend hook**: `src/features/students/hooks/useAttendanceThreshold.ts`

```typescript
import { invoke } from "@tauri-apps/api/core";

interface ThresholdResponse {
  success: boolean;
  data?: { value: number };
  error?: string;
}

export function useAttendanceThreshold() {
  const [threshold, setThreshold] = useState<number>(3); // default until fetched
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    invoke<ThresholdResponse>("get_absence_threshold").then((res) => {
      if (res.success && res.data) setThreshold(res.data.value);
      setIsLoading(false);
    });
  }, []);

  const updateThreshold = async (value: number) => {
    const validated = Math.max(1, Math.min(30, value));
    const res = await invoke<ThresholdResponse>("set_absence_threshold", {
      value: validated,
    });
    if (res.success && res.data) setThreshold(res.data.value);
    return res;
  };

  return { threshold, updateThreshold, isLoading };
}
```

### 2.4 WarningBadge Component

File: `src/features/students/components/WarningBadge.tsx`

```typescript
interface WarningBadgeProps {
  absenceCount: number;
  threshold: number;
}
```

- Uses existing `Badge` component with `variant="danger"`
- Shows: `"Más de {threshold} faltas ({count})"`
- Returns null if `absenceCount <= threshold`

### 2.5 AttendanceThresholdSettings Component

File: `src/features/students/components/AttendanceThresholdSettings.tsx`

- Number input with min=1, max=30
- Save button calls `set_absence_threshold` via the `useAttendanceThreshold` hook
- Shows current threshold value
- Inline or modal display
- Displays success/error toast after save

### 2.6 AtRiskStudentsWidget Component

File: `src/features/groups/components/AtRiskStudentsWidget.tsx`

```typescript
interface AtRiskStudentsWidgetProps {
  groupId: string;
  students: Student[];
  threshold: number;
  onRefresh?: () => void;
}
```

Features:
- Fetches group absence counts on mount and after attendance save
- Filters students where absenceCount > threshold
- Displays table: Student Name | Absences | Status badge
- Empty state: "No hay estudiantes en riesgo"
- Collapsible section with header count

### 2.7 Integration Points

**StudentList.tsx** — Add optional `absenceCounts` prop:
```typescript
interface StudentListProps {
  students: Student[];
  absenceCounts?: Map<string, number>;
  threshold?: number;
  onEdit?: (student: Student) => void;
  onDelete?: (student: Student) => void;
}
```

Add column after "Fecha registro":
```typescript
{
  key: "absences",
  header: "Asistencia",
  render: (student: Student) => {
    const count = absenceCounts?.get(student.id) || 0;
    return <WarningBadge absenceCount={count} threshold={threshold || 3} />;
  },
}
```

**GroupDetailInline.tsx** — Add AtRiskStudentsWidget:
```tsx
{/* Inside "Pasar Lista" tab, below DailyAttendanceForm */}
<AtRiskStudentsWidget
  groupId={groupId}
  students={groupStudents}
  threshold={threshold}
  onRefresh={() => { /* trigger re-fetch */ }}
/>
```

## 3. File-by-File Changes

### Backend (Rust)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/migrations/017_add_app_settings.sql` | Create | New `app_settings` table with default threshold |
| `src-tauri/src/application/ports/attendance.rs` | Modify | Add 2 trait methods for absence counting |
| `src-tauri/src/application/ports/settings.rs` | Create | Settings repository port (get/set key-value) |
| `src-tauri/src/infrastructure/repositories/sqlite/attendance.rs` | Modify | Implement 2 new queries |
| `src-tauri/src/infrastructure/repositories/sqlite/settings.rs` | Create | SQLite settings repository |
| `src-tauri/src/application/dto/attendance.rs` | Modify | Add `StudentAbsenceCount` DTO |
| `src-tauri/src/application/use_cases/attendance.rs` | Modify | Add 2 service methods |
| `src-tauri/src/application/use_cases/settings.rs` | Create | Settings use case (threshold get/set) |
| `src-tauri/src/commands/attendance.rs` | Modify | Add 2 commands + response types |
| `src-tauri/src/commands/settings.rs` | Create | `get_absence_threshold` + `set_absence_threshold` commands |
| `src-tauri/src/lib.rs` | Modify | Register 4 new commands, add migration, create SettingsService |

### Frontend (TypeScript/React)

| File | Action | Description |
|------|--------|-------------|
| `src/shared/types/AttendanceWarning.ts` | Create | `StudentAbsenceCount`, `AbsenceThreshold` types |
| `src/features/students/hooks/useStudentAbsences.ts` | Create | Hook for fetching absence data |
| `src/features/students/hooks/useAttendanceThreshold.ts` | Create | Hook wrapping `get/set_absence_threshold` commands |
| `src/features/students/components/AttendanceWarningBadge.tsx` | Create | Warning badge component |
| `src/features/students/components/AttendanceThresholdSettings.tsx` | Create | Threshold settings UI (calls DB commands) |
| `src/features/students/components/StudentList.tsx` | Modify | Add absence column |
| `src/features/groups/components/AtRiskStudentsWidget.tsx` | Create | At-risk students widget |
| `src/features/groups/components/GroupDetailInline.tsx` | Modify | Integrate widget |
| `src/features/groups/components/DailyAttendanceForm.tsx` | Modify | Trigger refresh on widget |

## 4. Implementation Order

### Phase 1: Backend (Foundation)
1. Create migration `017_add_app_settings.sql` with `app_settings` table
2. Add settings port, repository, use case, and commands (`get/set_absence_threshold`)
3. Add trait methods to `attendance.rs` port
4. Implement SQLite queries in repository
5. Add `StudentAbsenceCount` DTO
6. Add service methods to use case
7. Add Tauri commands + response types
8. Register all 4 commands in `lib.rs`
9. Test with `cargo check`

### Phase 2: Frontend Hooks & Types
1. Add types to `Attendance.ts`
2. Create `useStudentAbsences.ts` hook
3. Create `useAttendanceThreshold.ts` hook

### Phase 3: Components
1. Create `WarningBadge.tsx`
2. Create `AttendanceThresholdSettings.tsx`
3. Create `AtRiskStudentsWidget.tsx`

### Phase 4: Integration
1. Modify `StudentList.tsx` to show badges
2. Modify `GroupDetailInline.tsx` to show widget
3. Modify `DailyAttendanceForm.tsx` to trigger refresh

### Phase 5: Polish
1. Add `AttendanceThresholdSettings` to a settings area (TBD)
2. Test end-to-end flow
3. Run `bunx tsc --noEmit` and `bun run build`

## 5. Risks

| Risk | Mitigation |
|------|------------|
| Performance with many students | Batch query in `count_group_absences`, single query returns only counts |
| Threshold not found in DB | Default value (3) inserted via migration, use case returns 3 if row missing |
| Badge visual inconsistency | Reuses existing Badge component |
| Widget refresh after attendance | Pass `onRefresh` callback from parent |
| Settings service state management | Single `SettingsServiceState` managed via `tauri::async_runtime` |
