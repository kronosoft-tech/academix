# Groups Specification - Course Duration and Period

> Specification for groups module with course duration and scheduling features
> Last updated: 2026-07-16 (from change: course-duration-and-group-period)

---

## Overview

This specification covers the groups module enhancements for course duration integration, class scheduling, and end date calculation. The system connects course duration (total hours) with group scheduling by adding class duration fields and computing end dates automatically.

## Core Features

### 1. Class Duration Configuration
- Store class duration in minutes per session
- Validate class duration is positive when provided
- Allow null/undefined for backward compatibility

### 2. Automatic End Date Calculation
- Compute end date from start date, class duration, weekly schedule, and course duration
- Handle skipped dates (holidays) in calculation
- Display calculated end date in real-time

### 3. Skipped Dates Management
- Store skipped dates as JSON array
- Allow adding/removing specific dates
- Adjust end date calculation based on skipped dates

### 4. Date Display and Validation
- Show start date and calculated end date in group views
- Validate date relationships (start before end)
- Handle edge cases gracefully

## Data Model Changes

### groups_table modifications

```sql
-- Add class_duration column (minutes per session)
ALTER TABLE groups_table ADD COLUMN class_duration INTEGER;

-- Add skipped_dates column (JSON array of date strings)
ALTER TABLE groups_table ADD COLUMN skipped_dates TEXT DEFAULT '[]';
```

**Field specifications:**

| Field | Type | Constraints | Default | Description |
|-------|------|-------------|---------|-------------|
| class_duration | INTEGER | Nullable, positive when provided | NULL | Minutes per class session |
| skipped_dates | TEXT | JSON array format | '[]' | Dates to skip in calculation |

**Existing fields used:**

| Field | Type | Usage |
|-------|------|-------|
| start_date | TEXT | ISO date string, used as calculation baseline |
| days | TEXT | JSON array of weekday numbers (0-6, Sunday=0) |
| course_id | TEXT | Foreign key to courses table |

**Related course field:**

| Field | Type | Usage |
|-------|------|-------|
| courses.duration | INTEGER | Total course hours |

## Business Rules

### End Date Calculation Formula

```
1. total_sessions_needed = ceil((course_duration_hours × 60) / class_duration_minutes)
2. sessions_per_week = count(days_array)
3. weeks_needed = ceil(total_sessions_needed / sessions_per_week)
4. effective_weeks = weeks_needed + count(skipped_dates within period)
5. end_date = start_date + effective_weeks weeks (adjusted for day-of-week)
```

**Edge Cases:**

- **course_duration = 0**: Display "Sin duración definida" instead of calculated date
- **class_duration = 0 or null**: Disable auto-calculation, show "Definir duración de clase"
- **start_date not set**: Disable auto-calculation, show "Definir fecha de inicio"
- **days array empty**: Disable auto-calculation, show "Definir días de clase"
- **skipped_dates before start_date**: Ignore these dates
- **skipped_dates after calculated end_date**: Ignore these dates

### Skipped Dates Handling

- Store as ISO date strings in JSON array: `["2026-12-25", "2026-01-01"]`
- Dates are excluded from session count but extend calendar duration
- Only dates within the calculated period affect the result
- UI must allow adding/removing dates with date picker

### Validation Rules

1. `class_duration` must be > 0 when provided
2. `start_date` must be valid ISO date format
3. `skipped_dates` must be valid JSON array of ISO date strings
4. End date must be after start date when calculated
5. Days array must contain valid weekday numbers (0-6)

## API Commands

### New Commands

| Command | Description | Parameters |
|---------|-------------|------------|
| `calculate_group_end_date` | Compute end date for group | group_id |
| `add_skipped_date` | Add date to skip list | group_id, date |
| `remove_skipped_date` | Remove date from skip list | group_id, date |
| `get_group_schedule_summary` | Get schedule details with calculation | group_id |

### Modified Commands

| Command | Changes |
|---------|---------|
| `create_group` | Add class_duration, skipped_dates parameters |
| `update_group` | Add class_duration, skipped_dates parameters |
| `get_group` | Include calculated end_date in response |

## UI Changes

### Group Form (Create/Edit)

**New fields:**

| Field | Type | Validation | Description |
|-------|------|------------|-------------|
| class_duration | Number input | Min: 1, Step: 5 | Minutes per class session |
| skipped_dates | Date picker + list | Valid ISO dates | Dates to skip |

**Field behavior:**

- `class_duration`: Show when course has duration > 0
- `skipped_dates`: Show only when start_date is set
- Auto-calculate end_date on any field change
- Show loading indicator during calculation

### Group Display (Detail View)

**New display elements:**

| Element | Location | Description |
|---------|----------|-------------|
| Duration info | Header section | "Duración por clase: X minutos" |
| Schedule summary | Schedule section | "X sesiones por semana, Y semanas totales" |
| End date | Header section | "Fecha de fin calculada: YYYY-MM-DD" |
| Skipped dates list | Schedule section | List of skipped dates with remove button |

**Display logic:**

- Show "Sin duración definida" when course duration = 0
- Show "Definir duración de clase" when class_duration is null
- Show "Definir fecha de inicio" when start_date is null
- Show end date in bold when calculated, gray when estimated

### Group List/Table

**New columns (optional):**

| Column | Description |
|--------|-------------|
| End Date | Calculated end date |
| Duration | Class duration in minutes |

## Scenarios

### Scenario: Calculate end date with valid data

- GIVEN a group with start_date "2026-08-01", class_duration 60, days ["1", "3"] (Monday, Wednesday)
- AND linked course with duration 48 hours
- WHEN calculating end date
- THEN total_sessions_needed = ceil((48 × 60) / 60) = 48 sessions
- AND sessions_per_week = 2
- AND weeks_needed = ceil(48 / 2) = 24 weeks
- AND end_date = "2027-01-16" (24 weeks from start)

### Scenario: Calculate with skipped dates

- GIVEN same group as above
- AND skipped_dates ["2026-12-25", "2026-01-01"]
- WHEN calculating end date
- THEN effective_weeks = 24 + 1 (only one skipped date within period)
- AND end_date adjusts accordingly

### Scenario: Course duration is zero

- GIVEN a group with course having duration = 0
- WHEN displaying group details
- THEN show "Sin duración definida" instead of calculated end date
- AND disable auto-calculation

### Scenario: Class duration not set

- GIVEN a group with class_duration = null
- WHEN displaying group form
- THEN show "Definir duración de clase" placeholder
- AND disable end date calculation

### Scenario: Start date not set

- GIVEN a group with start_date = null
- WHEN displaying group form
- THEN show "Definir fecha de inicio" placeholder
- AND disable end date calculation

### Scenario: Add skipped date

- GIVEN a group with calculated end_date
- WHEN adding a skipped date within the period
- THEN end_date recalculates and extends by appropriate amount
- AND skipped date appears in the list

### Scenario: Remove skipped date

- GIVEN a group with skipped dates
- WHEN removing a skipped date
- THEN end_date recalculates and shortens by appropriate amount
- AND skipped date removed from list

### Scenario: Skipped date outside period

- GIVEN a group with calculated end_date "2026-12-15"
- WHEN adding skipped date "2027-01-01" (after end date)
- THEN skipped date is ignored in calculation
- AND end_date remains unchanged

### Scenario: Invalid class duration

- GIVEN a group form
- WHEN entering class_duration = 0 or negative
- THEN show validation error "La duración debe ser mayor a 0"
- AND prevent form submission

### Scenario: No weekly sessions defined

- GIVEN a group with days array empty
- WHEN calculating end date
- THEN show "Definir días de clase"
- AND disable calculation

## Acceptance Criteria

- [ ] `class_duration` field exists in groups_table with correct type
- [ ] `skipped_dates` field exists in groups_table with JSON default
- [ ] End date calculation matches formula for all valid inputs
- [ ] Edge cases display correct placeholder messages
- [ ] Skipped dates are stored and retrieved correctly
- [ ] UI updates end date in real-time when fields change
- [ ] Validation prevents invalid class duration values
- [ ] Skipped dates can be added and removed via UI
- [ ] End date adjusts when skipped dates are added/removed
- [ ] Dates outside calculation period are ignored
- [ ] Group list shows end date column (optional)
- [ ] Group detail shows schedule summary
- [ ] Form disables calculation when required fields missing
- [ ] All scenarios from this spec are testable
- [ ] No regression in existing group functionality