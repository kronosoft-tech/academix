## Spec: course-duration-and-group-period

### Executive Summary
This change connects course duration (total hours) with group scheduling by adding a `class_duration` field and computing `end_date` automatically. It enables precise session planning and exposes start/end dates in the frontend, improving scheduling accuracy and user experience.

### Requirements
- REQ-001: Add `class_duration` field (minutes per session) to groups table
- REQ-002: Add `skipped_dates` field (JSON array) to groups table
- REQ-003: Compute `end_date` automatically from start_date, class_duration, days, and course duration
- REQ-004: Handle skipped dates in end_date calculation
- REQ-005: Expose start_date and end_date in frontend group forms and views
- REQ-006: Validate class_duration is positive when provided
- REQ-007: Handle edge cases (course duration=0, class duration=null, start_date not set)
- REQ-008: Allow adding/removing skipped dates via UI

### Scenarios
**Scenario: Calculate end date with valid data**
- Given a group with start_date "2026-08-01", class_duration 60, days ["1", "3"] (Monday, Wednesday)
- And linked course with duration 48 hours
- When calculating end date
- Then total_sessions_needed = ceil((48 × 60) / 60) = 48 sessions
- And sessions_per_week = 2
- And weeks_needed = ceil(48 / 2) = 24 weeks
- And end_date = "2027-01-16" (24 weeks from start)

**Scenario: Calculate with skipped dates**
- Given same group as above
- And skipped_dates ["2026-12-25", "2026-01-01"]
- When calculating end date
- Then effective_weeks = 24 + 1 (only one skipped date within period)
- And end_date adjusts accordingly

**Scenario: Course duration is zero**
- Given a group with course having duration = 0
- When displaying group details
- Then show "Sin duración definida" instead of calculated end date
- And disable auto-calculation

**Scenario: Class duration not set**
- Given a group with class_duration = null
- When displaying group form
- Then show "Definir duración de clase" placeholder
- And disable end date calculation

**Scenario: Start date not set**
- Given a group with start_date = null
- When displaying group form
- Then show "Definir fecha de inicio" placeholder
- And disable end date calculation

**Scenario: Add skipped date**
- Given a group with calculated end_date
- When adding a skipped date within the period
- Then end_date recalculates and extends by appropriate amount
- And skipped date appears in the list

**Scenario: Remove skipped date**
- Given a group with skipped dates
- When removing a skipped date
- Then end_date recalculates and shortens by appropriate amount
- And skipped date removed from list

**Scenario: Skipped date outside period**
- Given a group with calculated end_date "2026-12-15"
- When adding skipped date "2027-01-01" (after end date)
- Then skipped date is ignored in calculation
- And end_date remains unchanged

**Scenario: Invalid class duration**
- Given a group form
- When entering class_duration = 0 or negative
- Then show validation error "La duración debe ser mayor a 0"
- And prevent form submission

**Scenario: No weekly sessions defined**
- Given a group with days array empty
- When calculating end date
- Then show "Definir días de clase"
- And disable calculation

### Data Model Changes
**New fields in groups_table:**
- `class_duration` INTEGER (nullable) - Minutes per class session
- `skipped_dates` TEXT DEFAULT '[]' - JSON array of ISO date strings to skip

**Existing fields used:**
- `start_date` TEXT - ISO date string baseline
- `days` TEXT - JSON array of weekday numbers (0-6)
- `course_id` TEXT - Foreign key to courses

**Related course field:**
- `courses.duration` INTEGER - Total course hours

### API Changes
**New commands:**
- `calculate_group_end_date(group_id)` - Compute end date for group
- `add_skipped_date(group_id, date)` - Add date to skip list
- `remove_skipped_date(group_id, date)` - Remove date from skip list
- `get_group_schedule_summary(group_id)` - Get schedule details with calculation

**Modified commands:**
- `create_group` - Add class_duration, skipped_dates parameters
- `update_group` - Add class_duration, skipped_dates parameters
- `get_group` - Include calculated end_date in response

### UI Changes
**Group Form (Create/Edit):**
- New field: class_duration (Number input, min: 1, step: 5)
- New field: skipped_dates (Date picker + list)
- Auto-calculate end_date on field changes
- Show loading indicator during calculation

**Group Detail View:**
- Duration info: "Duración por clase: X minutos"
- Schedule summary: "X sesiones por semana, Y semanas totales"
- End date display: "Fecha de fin calculada: YYYY-MM-DD"
- Skipped dates list with remove button

**Group List/Table:**
- Optional column: End Date (calculated)
- Optional column: Duration (class duration in minutes)

### Business Rules
**End Date Calculation Formula:**
1. total_sessions_needed = ceil((course_duration_hours × 60) / class_duration_minutes)
2. sessions_per_week = count(days_array)
3. weeks_needed = ceil(total_sessions_needed / sessions_per_week)
4. effective_weeks = weeks_needed + count(skipped_dates within period)
5. end_date = start_date + effective_weeks weeks (adjusted for day-of-week)

**Edge Cases:**
- course_duration = 0 → "Sin duración definida"
- class_duration = 0 or null → "Definir duración de clase"
- start_date not set → "Definir fecha de inicio"
- days array empty → "Definir días de clase"
- skipped_dates before start_date → ignore
- skipped_dates after calculated end_date → ignore

**Validation Rules:**
1. class_duration must be > 0 when provided
2. start_date must be valid ISO date format
3. skipped_dates must be valid JSON array of ISO date strings
4. End date must be after start date when calculated
5. Days array must contain valid weekday numbers (0-6)

### Acceptance Criteria
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