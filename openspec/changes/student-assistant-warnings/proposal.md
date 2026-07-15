# Proposal: Student Attendance Warnings

## Intent

The Academix system currently tracks student attendance per group but does not provide visibility into individual student absence patterns. Teachers and administrators need to quickly identify students who are frequently absent from specific groups to intervene early.

The goal is to **add warning badges** that show when a student has exceeded a configurable absence threshold in a specific group, displayed in both the student list table and the student detail modal. Additionally, teachers will see a dashboard widget listing at-risk students directly in the group attendance page where they take daily attendance.

## Scope

### In Scope
1. **Backend Command**: New Rust command to count absences per student per group
2. **Frontend Hook**: React hook to fetch and cache absence counts
3. **Warning Badge in Table**: Visual indicator in StudentsPage when absences exceed threshold
4. **Warning Badge in Modal**: Absence count display in student detail modal
5. **Spanish Text**: All warning messages in Spanish (e.g., "Más de X faltas", "Advertencia")
6. **Configurable Warning Threshold**: Users can set the absence count threshold (default: 3) via a settings component
7. **At-Risk Students Widget**: Dashboard in the Groups attendance page listing students who have exceeded the warning threshold

### Out of Scope
- Email/SMS notifications for absences
- Attendance reports or analytics
- Bulk actions on students with warnings
- Historical absence trends or charts

## Approach

### Step 1: Backend - Absence Count Command
Create a new Tauri command `get_student_absence_count` that:
- Takes `student_id` and `group_id` as parameters
- Queries the `attendance` table for records with status = 'absent'
- Returns the count of absences for that student in that group
- Handles edge cases (no attendance records, invalid IDs)

### Step 2: Backend - Batch Absence Count
Create a command `get_students_absence_counts` that:
- Takes a list of student IDs and a group ID
- Returns a map of student_id -> absence_count
- Optimized for bulk loading in the student list view

### Step 3: Frontend - useStudentAbsences Hook
Create a custom hook that:
- Wraps the Tauri command calls
- Provides loading states and error handling
- Caches results to avoid repeated calls
- Returns absence count and whether the configurable warning threshold is exceeded

### Step 4: Frontend - Warning Badge Component
Create a `WarningBadge` component that:
- Shows red/orange badge when absences exceed the configurable threshold
- Displays Spanish text with dynamic count (e.g., "Más de {threshold} faltas" or "Advertencia")
- Follows existing Badge component patterns (variants: danger, warning)

### Step 5: Frontend - Threshold Settings Component
Create an `AttendanceThresholdSettings` component that:
- Allows users to configure the absence warning threshold (default: 3)
- Persists the setting (local storage or backend config)
- Displays in a settings area or as an inline configuration option
- Validates minimum value of 1

### Step 6: Frontend - Integration in StudentsPage
Integrate the warning badge into:
- Student list table: Add absence count column with warning badge
- Student detail modal: Show absence information in student details section

### Step 7: Frontend - At-Risk Students Widget
Create an `AtRiskStudentsWidget` component for the Groups attendance page:
- Lists students in the current group who exceed the warning threshold
- Shows student name, absence count, and severity indicator
- Integrated into `DailyAttendanceForm.tsx` or `GroupDetailPage.tsx`
- Updates in real-time as attendance is taken
- Displays "No hay estudiantes en riesgo" when list is empty

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/src/commands/students.rs` | Modified | Add `get_student_absence_count` and `get_students_absence_counts` commands |
| `src-tauri/src/lib.rs` | Modified | Register new commands in `generate_handler!` |
| `src/features/students/hooks/useStudentAbsences.ts` | Created | New hook for fetching absence data |
| `src/features/students/components/WarningBadge.tsx` | Created | New component for absence warnings |
| `src/features/students/components/AttendanceThresholdSettings.tsx` | Created | Settings component for configurable threshold |
| `src/features/groups/components/AtRiskStudentsWidget.tsx` | Created | Widget showing students with warnings in groups attendance page |
| `src/features/groups/components/DailyAttendanceForm.tsx` | Modified | Integrate at-risk students widget |
| `src/features/groups/components/GroupDetailPage.tsx` | Modified | Integrate at-risk students widget |
| `src/features/students/components/StudentsTable.tsx` | Modified | Add absence count column with badge |
| `src/features/students/components/StudentDetailModal.tsx` | Modified | Add absence info section |
| `src/features/students/types/student.ts` | Modified | Add AbsenceCount type |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Performance impact on large student lists | Medium | Use batch command, implement pagination if needed |
| Incorrect absence counting | Low | Write unit tests for SQL query, verify status values |
| Badge styling inconsistent with design | Low | Reuse existing Badge component patterns |
| Spanish text not matching user expectations | Low | Use exact text from requirements |
| Widget performance with many at-risk students | Low | Implement virtual scrolling if list grows large |

## Rollback Plan

1. Revert all file changes (git checkout)
2. No database migrations needed (read-only queries)
3. No breaking changes to existing functionality

## Dependencies

- Existing `attendance` table with `student_id`, `group_id`, `date`, `status` columns
- Existing `Badge` component with danger/warning variants
- Tauri v2 command registration pattern

## Success Criteria

- [ ] Backend command returns correct absence count per student per group
- [ ] Frontend hook provides absence data with loading states
- [ ] Warning badge displays in student table when absences exceed threshold
- [ ] Warning badge displays in student detail modal
- [ ] All warning text in Spanish ("Más de {threshold} faltas", "Advertencia")
- [ ] Threshold is configurable with default value of 3
- [ ] Groups attendance page shows at-risk students widget listing students above threshold
- [ ] No performance degradation on student list page
- [ ] Unit tests for absence counting logic
