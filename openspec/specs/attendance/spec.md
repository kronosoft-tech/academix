# Spec: Attendance Module

> Main spec for attendance functionality
> Last updated: 2026-07-14 (from change: student-assistant-warnings)

---

## Overview

The attendance module tracks student presence in groups and provides warnings when students exceed a configurable absence threshold.

## Core Features

### 1. Attendance Recording
- Record student attendance per group per date
- Status values: `present`, `absent`, `late`, `excused`
- Unique constraint: one record per student per group per date

### 2. Absence Counting
- Count absences per student per group (status = 'absent' only)
- Batch counting for all students in a group
- Computed on-demand via SQL queries

### 3. Warning System
- Configurable threshold (default: 3 absences)
- Warning badge: "Más de X faltas" when count > threshold
- Displayed in:
  - StudentsPage table (per student)
  - StudentDetailModal (per group)
  - AtRiskStudentsWidget (group attendance page)

### 4. Threshold Configuration
- Stored in `app_settings` table
- Range: 1-30
- Accessible from StudentsPage

## Data Model

### attendance table
```sql
CREATE TABLE attendance (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL,
    group_id TEXT NOT NULL,
    date TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('present', 'absent', 'late', 'excused')),
    notes TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (student_id) REFERENCES students(id),
    FOREIGN KEY (group_id) REFERENCES groups_table(id),
    UNIQUE(student_id, group_id, date)
);
```

### app_settings table
```sql
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## API Commands

| Command | Description |
|---------|-------------|
| `count_student_absences` | Count absences for student in group |
| `count_group_absences` | Batch count for all students in group |
| `get_absence_threshold` | Get current threshold value |
| `set_absence_threshold` | Update threshold (1-30) |

## UI Components

| Component | Location | Description |
|-----------|----------|-------------|
| AttendanceWarningBadge | StudentsPage, StudentDetailModal | Badge showing warning when threshold exceeded |
| AttendanceThresholdSettings | StudentsPage | Configurable threshold input |
| AtRiskStudentsWidget | GroupDetailPage | List of at-risk students |
