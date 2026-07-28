//! MemoryBuffer-backed Group Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::GroupRepository;
use crate::domain::entities::group::{Group, GroupStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedGroupRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedGroupRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(group: &Group) -> CachedEntity {
        CachedEntity {
            id: group.id.clone(),
            data: HashMap::from([
                ("id".to_string(), group.id.clone()),
                ("course_id".to_string(), group.course_id.clone()),
                ("name".to_string(), group.name.clone()),
                ("professor_id".to_string(), group.professor_id.clone().unwrap_or_default()),
                ("schedule".to_string(), group.schedule.clone().unwrap_or_default()),
                ("days".to_string(), group.days.as_ref().map(|d| serde_json::to_string(d).unwrap_or_default()).unwrap_or_default()),
                ("start_time".to_string(), group.start_time.clone().unwrap_or_default()),
                ("end_time".to_string(), group.end_time.clone().unwrap_or_default()),
                ("start_date".to_string(), group.start_date.clone().unwrap_or_default()),
                ("end_date".to_string(), group.end_date.clone().unwrap_or_default()),
                ("max_students".to_string(), group.max_students.to_string()),
                ("current_students".to_string(), group.current_students.to_string()),
                ("status".to_string(), group.status.as_str().to_string()),
                ("created_at".to_string(), group.created_at.to_rfc3339()),
                ("updated_at".to_string(), group.updated_at.to_rfc3339()),
                ("class_duration".to_string(), group.class_duration.map(|d| d.to_string()).unwrap_or_default()),
                ("skipped_dates".to_string(), serde_json::to_string(&group.skipped_dates).unwrap_or_default()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<Group> {
        Some(Group {
            id: cached.data.get("id")?.clone(),
            course_id: cached.data.get("course_id")?.clone(),
            name: cached.data.get("name")?.clone(),
            professor_id: Self::opt_string(cached, "professor_id"),
            schedule: Self::opt_string(cached, "schedule"),
            days: cached.data.get("days")
                .and_then(|s| if s.is_empty() { None } else { serde_json::from_str(s).ok() }),
            start_time: Self::opt_string(cached, "start_time"),
            end_time: Self::opt_string(cached, "end_time"),
            start_date: Self::opt_string(cached, "start_date"),
            end_date: Self::opt_string(cached, "end_date"),
            max_students: cached.data.get("max_students")?.parse().unwrap_or(0),
            current_students: cached.data.get("current_students")?.parse().unwrap_or(0),
            status: {
                let s = cached.data.get("status")?;
                GroupStatus::from_str(s).unwrap_or(GroupStatus::Open)
            },
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            updated_at: cached.data.get("updated_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            class_duration: cached.data.get("class_duration")
                .and_then(|s| if s.is_empty() { None } else { s.parse().ok() }),
            skipped_dates: cached.data.get("skipped_dates")
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
        })
    }

    fn opt_string(cached: &CachedEntity, key: &str) -> Option<String> {
        cached.data.get(key).and_then(|v| {
            if v.is_empty() { None } else { Some(v.clone()) }
        })
    }

    fn row_to_group(row: &rusqlite::Row<'_>) -> rusqlite::Result<Group> {
        let id: String = row.get(0)?;
        let course_id: String = row.get(1)?;
        let name: String = row.get(2)?;
        let professor_id: Option<String> = row.get(3)?;
        let schedule: Option<String> = row.get(4)?;
        let days_json: Option<String> = row.get(5)?;
        let start_time: Option<String> = row.get(6)?;
        let end_time: Option<String> = row.get(7)?;
        let start_date: Option<String> = row.get(8)?;
        let end_date: Option<String> = row.get(9)?;
        let max_students: i32 = row.get(10)?;
        let current_students: i32 = row.get(11)?;
        let status_str: String = row.get(12)?;
        let created_str: String = row.get(13)?;
        let updated_str: String = row.get(14)?;
        let class_duration: Option<i32> = row.get(15)?;
        let skipped_dates_json: Option<String> = row.get(16)?;

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

impl GroupRepository for MemoryBackedGroupRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Group>, DomainError> {
        let cache_key = format!("group:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_group) {
            Ok(group) => Ok(Some(group)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_course_id(&self, course_id: &str) -> Result<Vec<Group>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE course_id = ? ORDER BY name";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![course_id], Self::row_to_group)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn save(&self, group: &Group) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(group).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "groups_table".to_string(),
            data,
        });
        Ok(())
    }

    fn update(&self, group: &Group) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(group).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "groups_table".to_string(),
            id: group.id.clone(),
            data,
        });
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "groups_table".to_string(),
            id: id.to_string(),
            data: HashMap::from([
                ("status".to_string(), "closed".to_string()),
            ]),
        });
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Group>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table ORDER BY name";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_group)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_professor_id(&self, professor_id: &str) -> Result<Vec<Group>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, course_id, name, professor_id, schedule, days, 
                          start_time, end_time, start_date, end_date, 
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE professor_id = ? ORDER BY name";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![professor_id], Self::row_to_group)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn has_capacity(&self, group_id: &str) -> Result<bool, DomainError> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT max_students, current_students FROM groups_table WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let result = conn.query_row(sql, [group_id], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
        });

        match result {
            Ok((max, current)) => Ok(current < max),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn increment_students(&self, group_id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "groups_table".to_string(),
            id: group_id.to_string(),
            data: HashMap::from([
                ("__increment_students".to_string(), "1".to_string()),
            ]),
        });
        Ok(())
    }

    fn decrement_students(&self, group_id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "groups_table".to_string(),
            id: group_id.to_string(),
            data: HashMap::from([
                ("__decrement_students".to_string(), "1".to_string()),
            ]),
        });
        Ok(())
    }
}
