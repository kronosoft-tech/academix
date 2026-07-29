//! MemoryBacked Group Repository
//!
//! Implements GroupRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5b: 6 MemoryBacked repositories.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::GroupRepository;
use crate::domain::entities::group::{Group, GroupStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

/// MemoryBuffer-backed implementation of GroupRepository.
#[derive(Clone)]
pub struct MemoryBackedGroupRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedGroupRepository {
    pub fn new(
        connection_manager: Arc<Mutex<ConnectionManager>>,
        memory_buffer: Arc<Mutex<MemoryBuffer>>,
        session: Arc<Mutex<CurrentSession>>,
    ) -> Self {
        Self {
            connection_manager,
            memory_buffer,
            session,
        }
    }

    /// Convert a libsql::Row into a HashMap<String, String> for cache storage.
    /// Column indices must match the SELECT statement order used in queries.
    fn row_to_hash_map(row: &libsql::Row) -> Result<HashMap<String, String>, DomainError> {
        let mut map = HashMap::new();
        map.insert(
            "id".to_string(),
            row.get::<String>(0)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "course_id".to_string(),
            row.get::<String>(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "name".to_string(),
            row.get::<String>(2)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "professor_id".to_string(),
            row.get::<Option<String>>(3)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "schedule".to_string(),
            row.get::<Option<String>>(4)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "days".to_string(),
            row.get::<Option<String>>(5)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "start_time".to_string(),
            row.get::<Option<String>>(6)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "end_time".to_string(),
            row.get::<Option<String>>(7)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "start_date".to_string(),
            row.get::<Option<String>>(8)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "end_date".to_string(),
            row.get::<Option<String>>(9)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "max_students".to_string(),
            row.get::<i32>(10)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .to_string(),
        );
        map.insert(
            "current_students".to_string(),
            row.get::<i32>(11)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .to_string(),
        );
        map.insert(
            "status".to_string(),
            row.get::<String>(12)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "created_at".to_string(),
            row.get::<String>(13)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "updated_at".to_string(),
            row.get::<String>(14)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "class_duration".to_string(),
            row.get::<Option<i32>>(15)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .map(|v| v.to_string())
                .unwrap_or_default(),
        );
        map.insert(
            "skipped_dates".to_string(),
            row.get::<Option<String>>(16)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        Ok(map)
    }

    fn row_to_group(row: &libsql::Row) -> Result<Group, DomainError> {
        let id: String = row
            .get(0)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let course_id: String = row
            .get(1)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let name: String = row
            .get(2)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let professor_id: Option<String> = row
            .get(3)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let schedule: Option<String> = row
            .get(4)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let days_json: Option<String> = row
            .get(5)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let start_time: Option<String> = row
            .get(6)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let end_time: Option<String> = row
            .get(7)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let start_date: Option<String> = row
            .get(8)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let end_date: Option<String> = row
            .get(9)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let max_students: i32 = row
            .get(10)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let current_students: i32 = row
            .get(11)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let status_str: String = row
            .get(12)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row
            .get(13)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row
            .get(14)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let class_duration: Option<i32> = row
            .get(15)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let skipped_dates_json: Option<String> = row
            .get(16)
            .map_err(|e| DomainError::Database(e.to_string()))?;

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

    fn group_from_data(data: &HashMap<String, String>) -> Result<Group, DomainError> {
        let status_str = data
            .get("status")
            .ok_or_else(|| DomainError::Database("missing status".into()))?;
        let created_str = data
            .get("created_at")
            .ok_or_else(|| DomainError::Database("missing created_at".into()))?;
        let updated_str = data
            .get("updated_at")
            .ok_or_else(|| DomainError::Database("missing updated_at".into()))?;

        let days: Option<Vec<String>> = data
            .get("days")
            .and_then(|v| if v.is_empty() { None } else { Some(v.clone()) })
            .and_then(|s| serde_json::from_str(&s).ok());

        let skipped_dates: Vec<String> = data
            .get("skipped_dates")
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Ok(Group {
            id: data
                .get("id")
                .ok_or_else(|| DomainError::Database("missing id".into()))?
                .clone(),
            course_id: data
                .get("course_id")
                .ok_or_else(|| DomainError::Database("missing course_id".into()))?
                .clone(),
            name: data
                .get("name")
                .ok_or_else(|| DomainError::Database("missing name".into()))?
                .clone(),
            professor_id: data.get("professor_id").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            schedule: data.get("schedule").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            days,
            start_time: data.get("start_time").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            end_time: data.get("end_time").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            start_date: data.get("start_date").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            end_date: data.get("end_date").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            max_students: data
                .get("max_students")
                .ok_or_else(|| DomainError::Database("missing max_students".into()))?
                .parse()
                .unwrap_or(0),
            current_students: data
                .get("current_students")
                .ok_or_else(|| DomainError::Database("missing current_students".into()))?
                .parse()
                .unwrap_or(0),
            status: GroupStatus::from_str(status_str).unwrap_or(GroupStatus::Open),
            created_at: DateTime::parse_from_rfc3339(created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            class_duration: data.get("class_duration").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    v.parse().ok()
                }
            }),
            skipped_dates,
        })
    }

    fn to_data_map(group: &Group) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), group.id.clone());
        data.insert("course_id".to_string(), group.course_id.clone());
        data.insert("name".to_string(), group.name.clone());
        data.insert(
            "professor_id".to_string(),
            group.professor_id.clone().unwrap_or_default(),
        );
        data.insert(
            "schedule".to_string(),
            group.schedule.clone().unwrap_or_default(),
        );
        data.insert(
            "days".to_string(),
            group
                .days
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default())
                .unwrap_or_default(),
        );
        data.insert(
            "start_time".to_string(),
            group.start_time.clone().unwrap_or_default(),
        );
        data.insert(
            "end_time".to_string(),
            group.end_time.clone().unwrap_or_default(),
        );
        data.insert(
            "start_date".to_string(),
            group.start_date.clone().unwrap_or_default(),
        );
        data.insert(
            "end_date".to_string(),
            group.end_date.clone().unwrap_or_default(),
        );
        data.insert("max_students".to_string(), group.max_students.to_string());
        data.insert(
            "current_students".to_string(),
            group.current_students.to_string(),
        );
        data.insert("status".to_string(), group.status.as_str().to_string());
        data.insert("created_at".to_string(), group.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), group.updated_at.to_rfc3339());
        data.insert(
            "class_duration".to_string(),
            group
                .class_duration
                .map(|d| d.to_string())
                .unwrap_or_default(),
        );
        data.insert(
            "skipped_dates".to_string(),
            serde_json::to_string(&group.skipped_dates).unwrap_or_default(),
        );
        data
    }

    async fn get_user_id(&self) -> Result<String, DomainError> {
        let session = self.session.lock().await;
        session
            .user_id
            .clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))
    }

    async fn find_group_internal(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<Option<Group>, DomainError> {
        // Check pending buffer writes first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "groups_table", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Ok(Some(Self::group_from_data(data)?));
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "groups_table", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Ok(Some(Self::group_from_data(data)?));
                }
            }
            if buf.has_pending_delete(&user_id, "groups_table", id) {
                return Ok(None);
            }
        }

        // Check entity cache or query Turso
        let row_data: Option<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_entity(&user_id, "groups_table", id) {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql = "SELECT id, course_id, name, professor_id, schedule, days,
                                  start_time, end_time, start_date, end_date,
                                  max_students, current_students, status, created_at, updated_at,
                                  class_duration, skipped_dates
                           FROM groups_table WHERE id = ?1";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![id]).await?;
                let data = match rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    Some(row) => Some(Self::row_to_hash_map(&row)?),
                    None => None,
                };

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_entity(&user_id, "groups_table", id, data.clone());
                data
            }
        };

        match row_data {
            Some(data) => Ok(Some(Self::group_from_data(&data)?)),
            None => Ok(None),
        }
    }

    async fn query_turso(
        &self,
        user_id: &str,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<libsql::Rows, DomainError> {
        let db = {
            let cm = self.connection_manager.lock().await;
            cm.get_connection(user_id)
                .map(|c| c.db.clone())
                .ok_or_else(|| DomainError::Database("No connection for user".to_string()))?
        };
        let conn = db
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        conn.query(sql, params)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))
    }
}

#[async_trait]
impl GroupRepository for MemoryBackedGroupRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Group>, DomainError> {
        let user_id = self.get_user_id().await?;
        self.find_group_internal(&user_id, id).await
    }

    async fn find_by_course_id(&self, course_id: &str) -> Result<Vec<Group>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Read from Turso
        let sql = "SELECT id, course_id, name, professor_id, schedule, days,
                          start_time, end_time, start_date, end_date,
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE course_id = ?1 ORDER BY name";
        let mut rows = self
            .query_turso(&user_id, sql, libsql::params![course_id])
            .await?;

        let mut results: Vec<Group> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            results.push(Self::row_to_group(&row)?);
        }

        // Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "groups_table");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                if data.get("course_id").map(|v| v.as_str()) == Some(course_id) {
                    results.push(Self::group_from_data(data)?);
                }
            }
        }

        let pending_updates = buf.scan_pending_updates(&user_id, "groups_table");
        for op in &pending_updates {
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
                if let Some(pos) = results.iter().position(|g| g.id == *update_id) {
                    results[pos] = Self::group_from_data(data)?;
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "groups_table");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|g| g.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn find_by_professor_id(&self, professor_id: &str) -> Result<Vec<Group>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Read from Turso
        let sql = "SELECT id, course_id, name, professor_id, schedule, days,
                          start_time, end_time, start_date, end_date,
                          max_students, current_students, status, created_at, updated_at,
                          class_duration, skipped_dates
                   FROM groups_table WHERE professor_id = ?1 ORDER BY name";
        let mut rows = self
            .query_turso(&user_id, sql, libsql::params![professor_id])
            .await?;

        let mut results: Vec<Group> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            results.push(Self::row_to_group(&row)?);
        }

        // Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "groups_table");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                if data.get("professor_id").map(|v| v.as_str()) == Some(professor_id) {
                    results.push(Self::group_from_data(data)?);
                }
            }
        }

        let pending_updates = buf.scan_pending_updates(&user_id, "groups_table");
        for op in &pending_updates {
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
                if let Some(pos) = results.iter().position(|g| g.id == *update_id) {
                    results[pos] = Self::group_from_data(data)?;
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "groups_table");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|g| g.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn save(&self, group: &Group) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;
        let data = Self::to_data_map(group);
        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "groups_table".to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn update(&self, group: &Group) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut data = HashMap::new();
        data.insert("name".to_string(), group.name.clone());
        data.insert(
            "professor_id".to_string(),
            group.professor_id.clone().unwrap_or_default(),
        );
        data.insert(
            "schedule".to_string(),
            group.schedule.clone().unwrap_or_default(),
        );
        data.insert("max_students".to_string(), group.max_students.to_string());
        data.insert(
            "current_students".to_string(),
            group.current_students.to_string(),
        );
        data.insert("status".to_string(), group.status.as_str().to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert(
            "days".to_string(),
            group
                .days
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default())
                .unwrap_or_default(),
        );
        data.insert(
            "start_time".to_string(),
            group.start_time.clone().unwrap_or_default(),
        );
        data.insert(
            "end_time".to_string(),
            group.end_time.clone().unwrap_or_default(),
        );
        data.insert(
            "start_date".to_string(),
            group.start_date.clone().unwrap_or_default(),
        );
        data.insert(
            "end_date".to_string(),
            group.end_date.clone().unwrap_or_default(),
        );
        data.insert(
            "class_duration".to_string(),
            group
                .class_duration
                .map(|d| d.to_string())
                .unwrap_or_default(),
        );
        data.insert(
            "skipped_dates".to_string(),
            serde_json::to_string(&group.skipped_dates).unwrap_or_default(),
        );
        data.insert("id".to_string(), group.id.clone()); // for deserialization
        data.insert("course_id".to_string(), group.course_id.clone()); // for deserialization

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "groups_table".to_string(),
                id: group.id.clone(),
                data,
            },
        );
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        // Soft delete — set status to closed
        let mut data = HashMap::new();
        data.insert("status".to_string(), "closed".to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("id".to_string(), id.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "groups_table".to_string(),
                id: id.to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Group>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Step 1: Check cache or query Turso
        let base_rows: Vec<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_list(&user_id, "groups_table") {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql = "SELECT id, course_id, name, professor_id, schedule, days,
                                  start_time, end_time, start_date, end_date,
                                  max_students, current_students, status, created_at, updated_at,
                                  class_duration, skipped_dates
                           FROM groups_table ORDER BY name";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

                let mut raw_rows: Vec<HashMap<String, String>> = Vec::new();
                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    raw_rows.push(Self::row_to_hash_map(&row)?);
                }

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_list(&user_id, "groups_table", raw_rows.clone());
                raw_rows
            }
        };

        // Step 2: Convert rows to domain entities
        let mut results: Vec<Group> = base_rows
            .iter()
            .map(|data| Self::group_from_data(data))
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "groups_table");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                results.push(Self::group_from_data(data)?);
            }
        }

        let pending_updates = buf.scan_pending_updates(&user_id, "groups_table");
        for op in &pending_updates {
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
                if let Some(pos) = results.iter().position(|g| g.id == *update_id) {
                    results[pos] = Self::group_from_data(data)?;
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "groups_table");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|g| g.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn has_capacity(&self, group_id: &str) -> Result<bool, DomainError> {
        let user_id = self.get_user_id().await?;

        // Try to get the full group object from buffer or Turso
        if let Some(group) = self.find_group_internal(&user_id, group_id).await? {
            Ok(group.current_students < group.max_students && group.status == GroupStatus::Open)
        } else {
            Ok(false)
        }
    }

    async fn increment_students(&self, group_id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        // Read current current_students from Turso or pending buffer
        let current = {
            let buf = self.memory_buffer.lock().await;

            // Check if there's a pending insert with this group
            if let Some(op) = buf.find_pending_insert(&user_id, "groups_table", group_id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    let students: i32 = data
                        .get("current_students")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    Some(students)
                } else {
                    None
                }
            // Check if there's a pending update with this group
            } else if let Some(op) = buf.find_pending_update(&user_id, "groups_table", group_id) {
                if let BufferedOperation::Update { data, .. } = op {
                    let students: i32 = data
                        .get("current_students")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    Some(students)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let current_students = match current {
            Some(val) => val,
            None => {
                // Read from Turso
                let sql = "SELECT current_students FROM groups_table WHERE id = ?1";
                let mut rows = self
                    .query_turso(&user_id, sql, libsql::params![group_id])
                    .await?;
                match rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    Some(row) => row
                        .get(0)
                        .map_err(|e| DomainError::Database(e.to_string()))?,
                    None => return Err(DomainError::not_found("Group", group_id)),
                }
            }
        };

        let mut data = HashMap::new();
        data.insert(
            "current_students".to_string(),
            (current_students + 1).to_string(),
        );
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("id".to_string(), group_id.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "groups_table".to_string(),
                id: group_id.to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn decrement_students(&self, group_id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        // Read current current_students from Turso or pending buffer
        let current = {
            let buf = self.memory_buffer.lock().await;

            if let Some(op) = buf.find_pending_insert(&user_id, "groups_table", group_id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    data.get("current_students")
                        .and_then(|v| v.parse::<i32>().ok())
                } else {
                    None
                }
            } else if let Some(op) = buf.find_pending_update(&user_id, "groups_table", group_id) {
                if let BufferedOperation::Update { data, .. } = op {
                    data.get("current_students")
                        .and_then(|v| v.parse::<i32>().ok())
                } else {
                    None
                }
            } else {
                None
            }
        };

        let current_students = match current {
            Some(val) => val,
            None => {
                // Read from Turso
                let sql = "SELECT current_students FROM groups_table WHERE id = ?1";
                let mut rows = self
                    .query_turso(&user_id, sql, libsql::params![group_id])
                    .await?;
                match rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    Some(row) => row
                        .get(0)
                        .map_err(|e| DomainError::Database(e.to_string()))?,
                    None => return Err(DomainError::not_found("Group", group_id)),
                }
            }
        };

        let new_count = if current_students > 0 {
            current_students - 1
        } else {
            0
        };

        let mut data = HashMap::new();
        data.insert("current_students".to_string(), new_count.to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("id".to_string(), group_id.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "groups_table".to_string(),
                id: group_id.to_string(),
                data,
            },
        );
        Ok(())
    }
}
