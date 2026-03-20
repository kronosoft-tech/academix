//! Group SQLite Repository
//!
//! Implements GroupRepository using SQLite.

use crate::application::ports::GroupRepository;
use crate::domain::entities::group::{Group, GroupStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of GroupRepository
pub struct SqliteGroupRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteGroupRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_group(row: &rusqlite::Row<'_>) -> rusqlite::Result<Group> {
        let schedule: Option<String> = row.get(6)?;
        let status_str: String = row.get(7)?;
        let created_str: String = row.get(8)?;
        let updated_str: String = row.get(9)?;
        let days_json: Option<String> = row.get(10)?;
        let start_time: Option<String> = row.get(11)?;
        let end_time: Option<String> = row.get(12)?;

        // Parse days from JSON if present
        let days: Option<Vec<String>> = days_json.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Group {
            id: row.get(0)?,
            course_id: row.get(1)?,
            name: row.get(2)?,
            professor_id: row.get::<_, Option<String>>(3)?.filter(|s| !s.is_empty()),
            schedule: schedule.filter(|s| !s.is_empty()),
            days,
            start_time,
            end_time,
            max_students: row.get(4)?,
            current_students: row.get(5)?,
            status: GroupStatus::from_str(&status_str).unwrap_or(GroupStatus::Open),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl GroupRepository for SqliteGroupRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, max_students, current_students, 
                          schedule, status, created_at, updated_at, days, start_time, end_time
                   FROM groups_table WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_group)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_course_id(&self, course_id: &str) -> Result<Vec<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, max_students, current_students, 
                          schedule, status, created_at, updated_at, days, start_time, end_time
                   FROM groups_table WHERE course_id = ? ORDER BY name";

        self.pool
            .query(sql, &[&course_id], Self::row_to_group)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn save(&self, group: &Group) -> Result<(), DomainError> {
        let sql = "INSERT INTO groups_table (id, course_id, name, professor_id, max_students, 
                                     current_students, schedule, status, created_at, updated_at, days, start_time, end_time)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let days_json = group
            .days
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap_or_default());

        self.pool
            .execute(
                sql,
                &[
                    &group.id,
                    &group.course_id,
                    &group.name,
                    &group.professor_id,
                    &group.max_students.to_string(),
                    &group.current_students.to_string(),
                    &group.schedule,
                    &group.status.as_str().to_string(),
                    &group.created_at.to_rfc3339(),
                    &group.updated_at.to_rfc3339(),
                    &days_json,
                    &group.start_time,
                    &group.end_time,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn update(&self, group: &Group) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table 
                   SET name = ?, professor_id = ?, schedule = ?, max_students = ?, current_students = ?, 
                       status = ?, updated_at = ?, days = ?, start_time = ?, end_time = ?
                   WHERE id = ?";

        let days_json = group
            .days
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap_or_default());

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &group.name,
                    &group.professor_id,
                    &group.schedule,
                    &group.max_students.to_string(),
                    &group.current_students.to_string(),
                    &group.status.as_str().to_string(),
                    &Utc::now().to_rfc3339(),
                    &days_json,
                    &group.start_time,
                    &group.end_time,
                    &group.id,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", &group.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table SET status = 'closed', updated_at = ? WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&Utc::now().to_rfc3339(), &id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", id));
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, max_students, current_students, 
                          schedule, status, created_at, updated_at, days, start_time, end_time
                   FROM groups_table ORDER BY name";

        self.pool
            .query(sql, &[], Self::row_to_group)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_professor_id(&self, professor_id: &str) -> Result<Vec<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, max_students, current_students, 
                          schedule, status, created_at, updated_at, days, start_time, end_time
                   FROM groups_table WHERE professor_id = ? ORDER BY name";

        self.pool
            .query(sql, &[&professor_id], Self::row_to_group)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn has_capacity(&self, group_id: &str) -> Result<bool, DomainError> {
        let sql = "SELECT max_students, current_students FROM groups_table WHERE id = ?";

        let result = self.pool.query_row(sql, &[&group_id], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
        });

        match result {
            Ok(Some((max, current))) => Ok(current < max),
            Ok(None) => Ok(false),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn increment_students(&self, group_id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table SET current_students = current_students + 1 WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&group_id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", group_id));
        }
        Ok(())
    }

    fn decrement_students(&self, group_id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table SET current_students = current_students - 1 
                   WHERE id = ? AND current_students > 0";

        let affected = self
            .pool
            .execute(sql, &[&group_id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", group_id));
        }
        Ok(())
    }
}
