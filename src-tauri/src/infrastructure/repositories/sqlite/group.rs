//! Group SQLite Repository
//!
//! Implements GroupRepository using SQLite.

use async_trait::async_trait;
use crate::application::ports::GroupRepository;
use crate::domain::entities::group::{Group, GroupStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::local_db;
use chrono::{DateTime, Utc};

/// SQLite implementation of GroupRepository
#[derive(Clone)]
pub struct SqliteGroupRepository;

impl SqliteGroupRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_group(row: &libsql::Row) -> Result<Group, DomainError> {
        let id: String = row.get(0).map_err(|e| DomainError::Database(e.to_string()))?;
        let course_id: String = row.get(1).map_err(|e| DomainError::Database(e.to_string()))?;
        let name: String = row.get(2).map_err(|e| DomainError::Database(e.to_string()))?;
        let professor_id: Option<String> = row.get(3).map_err(|e| DomainError::Database(e.to_string()))?;
        let schedule: Option<String> = row.get(4).map_err(|e| DomainError::Database(e.to_string()))?;
        let days_json: Option<String> = row.get(5).map_err(|e| DomainError::Database(e.to_string()))?;
        let start_time: Option<String> = row.get(6).map_err(|e| DomainError::Database(e.to_string()))?;
        let end_time: Option<String> = row.get(7).map_err(|e| DomainError::Database(e.to_string()))?;
        let start_date: Option<String> = row.get(8).map_err(|e| DomainError::Database(e.to_string()))?;
        let end_date: Option<String> = row.get(9).map_err(|e| DomainError::Database(e.to_string()))?;
        let max_students: i32 = row.get(10).map_err(|e| DomainError::Database(e.to_string()))?;
        let current_students: i32 = row.get(11).map_err(|e| DomainError::Database(e.to_string()))?;
        let status_str: String = row.get(12).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(13).map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row.get(14).map_err(|e| DomainError::Database(e.to_string()))?;
        let class_duration: Option<i32> = row.get(15).map_err(|e| DomainError::Database(e.to_string()))?;
        let skipped_dates_json: Option<String> = row.get(16).map_err(|e| DomainError::Database(e.to_string()))?;

        let days: Option<Vec<String>> = days_json.and_then(|s| serde_json::from_str(&s).ok());
        let skipped_dates: Vec<String> = skipped_dates_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Ok(Group {
            id,
            course_id,
            name,
            professor_id,
            schedule,
            days,
            start_time,
            end_time,
            start_date,
            end_date,
            max_students,
            current_students,
            status: GroupStatus::from_str(&status_str).unwrap_or(GroupStatus::Open),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            class_duration,
            skipped_dates,
        })
    }
}

impl Default for SqliteGroupRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GroupRepository for SqliteGroupRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![id.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_group(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_course_id(&self, course_id: &str) -> Result<Vec<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE course_id = ?1 ORDER BY name";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![course_id.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_group(&row)?);
        }
        Ok(results)
    }

    async fn save(&self, group: &Group) -> Result<(), DomainError> {
        let sql = "INSERT INTO groups_table (id, course_id, name, professor_id, schedule, days, 
                                            start_time, end_time, start_date, end_date, 
                                            max_students, current_students, status, created_at, updated_at,
                                            class_duration, skipped_dates)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)";
        let days_json = group
            .days
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap_or_default());
        let skipped_dates_json = Some(serde_json::to_string(&group.skipped_dates).unwrap_or_default());

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(
            sql,
            libsql::params![
                group.id.clone(),
                group.course_id.clone(),
                group.name.clone(),
                group.professor_id.clone(),
                group.schedule.clone(),
                days_json,
                group.start_time.clone(),
                group.end_time.clone(),
                group.start_date.clone(),
                group.end_date.clone(),
                group.max_students.clone(),
                group.current_students.clone(),
                group.status.as_str().to_string(),
                group.created_at.to_rfc3339(),
                group.updated_at.to_rfc3339(),
                group.class_duration.clone(),
                skipped_dates_json,
            ],
        )
        .await
        .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, group: &Group) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table 
                   SET name = ?1, professor_id = ?2, schedule = ?3, max_students = ?4, current_students = ?5, 
                       status = ?6, updated_at = ?7, days = ?8, start_time = ?9, end_time = ?10, start_date = ?11, end_date = ?12,
                       class_duration = ?13, skipped_dates = ?14
                   WHERE id = ?15";

        let days_json = group
            .days
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap_or_default());
        let skipped_dates_json = Some(serde_json::to_string(&group.skipped_dates).unwrap_or_default());

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    group.name.clone(),
                    group.professor_id.clone(),
                    group.schedule.clone(),
                    group.max_students.clone(),
                    group.current_students.clone(),
                    group.status.as_str().to_string(),
                    Utc::now().to_rfc3339(),
                    days_json,
                    group.start_time.clone(),
                    group.end_time.clone(),
                    group.start_date.clone(),
                    group.end_date.clone(),
                    group.class_duration.clone(),
                    skipped_dates_json,
                    group.id.clone(),
                ],
            )
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", &group.id));
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table SET status = 'closed', updated_at = ?1 WHERE id = ?2";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![Utc::now().to_rfc3339(), id.to_string()])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", id));
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table ORDER BY name";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_group(&row)?);
        }
        Ok(results)
    }

    async fn find_by_professor_id(&self, professor_id: &str) -> Result<Vec<Group>, DomainError> {
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE professor_id = ?1 ORDER BY name";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![professor_id.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_group(&row)?);
        }
        Ok(results)
    }

    async fn has_capacity(&self, group_id: &str) -> Result<bool, DomainError> {
        let sql = "SELECT max_students, current_students FROM groups_table WHERE id = ?1";
        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![group_id.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => {
                let max: i32 = row.get(0).map_err(|e| DomainError::Database(e.to_string()))?;
                let current: i32 = row.get(1).map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(current < max)
            }
            None => Ok(false),
        }
    }

    async fn increment_students(&self, group_id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table SET current_students = current_students + 1 WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![group_id.to_string()])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", group_id));
        }
        Ok(())
    }

    async fn decrement_students(&self, group_id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE groups_table SET current_students = current_students - 1 
                   WHERE id = ?1 AND current_students > 0";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![group_id.to_string()])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Group", group_id));
        }
        Ok(())
    }
}
