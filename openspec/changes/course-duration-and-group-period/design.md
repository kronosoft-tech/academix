# Design: course-duration-and-group-period

## Technical Approach

This change adds `class_duration` and `skipped_dates` fields to groups, then computes `end_date` automatically based on course duration, class duration, weekly schedule, and skipped dates. The calculation logic lives in the domain layer as a pure function, with the application layer orchestrating data retrieval and the presentation layer handling real-time updates.

## Architecture Decisions

### Decision: Calculation Logic Location

**Choice**: Domain layer as a pure function on Group entity
**Alternatives considered**: 
- Application service method
- Frontend-only calculation
- Database trigger/computed column

**Rationale**: Domain layer keeps business logic close to the entity, is testable in isolation, and ensures consistency across all clients. Frontend-only would risk inconsistency; database trigger would be harder to maintain.

### Decision: Skipped Dates Storage Format

**Choice**: JSON array of ISO date strings in TEXT column
**Alternatives considered**:
- Separate `skipped_dates` table with foreign key
- Array of weekday numbers to skip weekly

**Rationale**: JSON array is simple, already used for `days` field, and sufficient for the use case. A separate table would add complexity without benefit since dates are not queried individually.

### Decision: End Date Calculation Strategy

**Choice**: Calculate on-demand when group is fetched or updated
**Alternatives considered**:
- Calculate on every save and store result
- Calculate only when displaying in UI
- Use database computed column

**Rationale**: On-demand calculation ensures the result is always current without requiring updates when course duration or schedule changes. Storing would risk stale data; UI-only would risk inconsistency.

## Data Flow

```
Frontend Form → Tauri Command → GroupService → Group Entity (calculation) → Repository → SQLite
     ↑                                                                    ↓
     └────────────────────────── GroupDto ←───────────────────────────────┘
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/migrations/018_add_class_duration_and_skipped_dates.sql` | Create | Add new columns to groups_table |
| `src-tauri/src/domain/entities/group.rs` | Modify | Add `class_duration`, `skipped_dates` fields and calculation method |
| `src-tauri/src/application/dto/group.rs` | Modify | Add new fields to DTOs |
| `src-tauri/src/application/use_cases/group.rs` | Modify | Handle new fields in create/update |
| `src-tauri/src/infrastructure/repositories/sqlite/group.rs` | Modify | Update SQL queries and row mapping |
| `src-tauri/src/commands/groups.rs` | Modify | Add new fields to command payloads |
| `src/shared/types/Group.ts` | Modify | Add TypeScript types for new fields |
| `src/features/groups/hooks/useGroups.ts` | Modify | Handle new fields in mapping |
| `src/features/groups/components/GroupForm.tsx` | Modify | Add class_duration and skipped_dates inputs |
| `src/features/groups/components/GroupDetailView.tsx` | Modify | Display calculated end_date and skipped dates |

## Interfaces / Contracts

### Rust Entity Changes

```rust
// In domain/entities/group.rs
pub struct Group {
    // ... existing fields ...
    pub class_duration: Option<i32>, // Minutes per session
    pub skipped_dates: Vec<String>, // ISO date strings to skip
}

impl Group {
    /// Calculate end date based on course duration, class duration, and schedule
    pub fn calculate_end_date(&self, course_duration_hours: i32) -> Option<String> {
        // Implementation in Calculation Logic section
    }
}
```

### DTO Changes

```rust
// In application/dto/group.rs
pub struct CreateGroupRequest {
    // ... existing fields ...
    pub class_duration: Option<i32>,
    pub skipped_dates: Option<Vec<String>>,
}

pub struct GroupDto {
    // ... existing fields ...
    pub class_duration: Option<i32>,
    pub skipped_dates: Vec<String>,
    pub calculated_end_date: Option<String>, // Computed field
}
```

### TypeScript Type Changes

```typescript
// In src/shared/types/Group.ts
export interface Group {
  // ... existing fields ...
  classDuration?: number;
  skippedDates?: string[];
  calculatedEndDate?: string; // Computed field
}

export interface CreateGroupInput {
  // ... existing fields ...
  classDuration?: number;
  skippedDates?: string[];
}
```

## Calculation Logic

### End Date Algorithm

```rust
pub fn calculate_end_date(&self, course_duration_hours: i32) -> Option<String> {
    // Edge cases
    if course_duration_hours <= 0 || self.class_duration.unwrap_or(0) <= 0 {
        return None;
    }
    
    let start_date = self.start_date.as_ref()?;
    let days = self.days.as_ref()?;
    
    if days.is_empty() {
        return None;
    }
    
    // Parse start date
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d").ok()?;
    
    // Calculate total sessions needed
    let total_minutes = course_duration_hours * 60;
    let class_duration = self.class_duration?;
    let total_sessions = (total_minutes as f64 / class_duration as f64).ceil() as i32;
    
    // Sessions per week
    let sessions_per_week = days.len() as i32;
    
    // Weeks needed (without skipped dates)
    let weeks_needed = (total_sessions as f64 / sessions_per_week as f64).ceil() as i32;
    
    // Calculate base end date
    let base_end = start + chrono::Duration::weeks(weeks_needed as i64);
    
    // Adjust for skipped dates within period
    let skipped_in_period = self.skipped_dates.iter()
        .filter_map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .filter(|d| *d >= start && *d < base_end)
        .count() as i32;
    
    let final_end = base_end + chrono::Duration::weeks(skipped_in_period as i64);
    
    Some(final_end.format("%Y-%m-%d").to_string())
}
```

### Edge Case Handling

1. **course_duration = 0**: Return `None`, display "Sin duración definida"
2. **class_duration = null**: Return `None`, display "Definir duración de clase"
3. **start_date = null**: Return `None`, display "Definir fecha de inicio"
4. **days empty**: Return `None`, display "Definir días de clase"
5. **skipped_dates before start**: Filter out in calculation
6. **skipped_dates after end**: Filter out in calculation

## Error Handling

### Validation Errors

- **class_duration ≤ 0**: "La duración debe ser mayor a 0"
- **invalid start_date format**: "Fecha de inicio inválida"
- **invalid skipped_dates format**: "Formato de fechas omitidas inválido"
- **end_date before start_date**: "La fecha de fin debe ser posterior a la fecha de inicio"

### Calculation Errors

- **Missing required fields**: Disable calculation, show placeholder
- **Invalid date parsing**: Log error, return None
- **Overflow in calculation**: Use i64 for intermediate calculations

### User Feedback

- Show loading indicator during calculation
- Display placeholder text for missing fields
- Highlight calculated end_date in bold
- Show validation errors inline

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | `Group::calculate_end_date()` with various inputs | Table-driven tests with edge cases |
| Unit | DTO serialization/deserialization | Serde roundtrip tests |
| Integration | Repository operations with new fields | SQLite integration tests |
| Integration | GroupService create/update with new fields | Mock repository tests |
| E2E | Group form with class_duration input | Playwright form interaction |
| E2E | End date calculation display | Playwright UI verification |

## Migration / Rollout

### Migration Strategy

```sql
-- 018_add_class_duration_and_skipped_dates.sql
ALTER TABLE groups_table ADD COLUMN class_duration INTEGER;
ALTER TABLE groups_table ADD COLUMN skipped_dates TEXT DEFAULT '[]';
```

- New columns are nullable for backward compatibility
- No data migration needed for existing groups
- Graceful degradation when fields are missing

### Rollout Plan

1. Deploy migration
2. Update backend code
3. Update frontend types
4. Update UI components
5. Test with existing groups (should show placeholders)
6. Test with new groups (should calculate end_date)

## Open Questions

- [ ] Should we cache calculated end_date in the database for performance?
- [ ] How should we handle timezone issues with date calculations?
- [ ] Should skipped_dates be limited to a maximum count?
- [ ] Should we add a "recalculate" button for manual refresh?

## Implementation Order

1. **Database migration** (018_add_class_duration_and_skipped_dates.sql)
2. **Domain entity changes** (group.rs - add fields and calculation method)
3. **DTO changes** (group.rs - add new fields)
4. **Repository changes** (sqlite/group.rs - update queries)
5. **Use case changes** (group.rs - handle new fields)
6. **Command changes** (groups.rs - add new fields to commands)
7. **Frontend type changes** (Group.ts)
8. **Frontend hook changes** (useGroups.ts)
9. **Frontend form changes** (GroupForm.tsx)
10. **Frontend display changes** (GroupDetailView.tsx)
11. **Unit tests** (calculation logic)
12. **Integration tests** (repository and service)
13. **E2E tests** (UI interaction)

## Relevant Files

- `src-tauri/src/domain/entities/group.rs` - Group entity with calculation logic
- `src-tauri/src/application/dto/group.rs` - DTO definitions
- `src-tauri/src/application/use_cases/group.rs` - Group service
- `src-tauri/src/infrastructure/repositories/sqlite/group.rs` - SQLite repository
- `src-tauri/src/commands/groups.rs` - Tauri commands
- `src/shared/types/Group.ts` - TypeScript types
- `src/features/groups/hooks/useGroups.ts` - Frontend hook
- `src/features/groups/components/GroupForm.tsx` - Group form component
- `src/features/groups/components/GroupDetailView.tsx` - Group detail view