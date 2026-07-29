//! MemoryBacked Course Repository
//!
//! Implements CourseRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5b: 6 MemoryBacked repositories.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::CourseRepository;
use crate::domain::entities::course::{Course, CourseStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

/// MemoryBuffer-backed implementation of CourseRepository.
#[derive(Clone)]
pub struct MemoryBackedCourseRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedCourseRepository {
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
            "name".to_string(),
            row.get::<String>(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "description".to_string(),
            row.get::<Option<String>>(2)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "code".to_string(),
            row.get::<String>(3)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "credits".to_string(),
            row.get::<i32>(4)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .to_string(),
        );
        map.insert(
            "status".to_string(),
            row.get::<String>(5)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "created_at".to_string(),
            row.get::<String>(6)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "updated_at".to_string(),
            row.get::<String>(7)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "price".to_string(),
            row.get::<f64>(8).unwrap_or(200000.0).to_string(),
        );
        map.insert(
            "duration".to_string(),
            row.get::<Option<i32>>(9)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or(0)
                .to_string(),
        );
        Ok(map)
    }

    fn row_to_course(row: &libsql::Row) -> Result<Course, DomainError> {
        let status_str: String = row
            .get(5)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row
            .get(6)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row
            .get(7)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let price: f64 = row.get(8).unwrap_or(200000.0);
        let duration: i32 = row
            .get::<Option<i32>>(9)
            .map_err(|e| DomainError::Database(e.to_string()))?
            .unwrap_or(0);

        Ok(Course {
            id: row
                .get(0)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            name: row
                .get(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            description: row
                .get(2)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            code: row
                .get(3)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            credits: row
                .get(4)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            price,
            duration,
            status: CourseStatus::from_str(&status_str).unwrap_or(CourseStatus::Draft),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn course_from_data(data: &HashMap<String, String>) -> Result<Course, DomainError> {
        let status_str = data
            .get("status")
            .ok_or_else(|| DomainError::Database("missing status".into()))?;
        let created_str = data
            .get("created_at")
            .ok_or_else(|| DomainError::Database("missing created_at".into()))?;
        let updated_str = data
            .get("updated_at")
            .ok_or_else(|| DomainError::Database("missing updated_at".into()))?;

        Ok(Course {
            id: data
                .get("id")
                .ok_or_else(|| DomainError::Database("missing id".into()))?
                .clone(),
            name: data
                .get("name")
                .ok_or_else(|| DomainError::Database("missing name".into()))?
                .clone(),
            description: data.get("description").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            code: data
                .get("code")
                .ok_or_else(|| DomainError::Database("missing code".into()))?
                .clone(),
            credits: data
                .get("credits")
                .ok_or_else(|| DomainError::Database("missing credits".into()))?
                .parse()
                .unwrap_or(0),
            price: data
                .get("price")
                .ok_or_else(|| DomainError::Database("missing price".into()))?
                .parse()
                .unwrap_or(200000.0),
            duration: data
                .get("duration")
                .ok_or_else(|| DomainError::Database("missing duration".into()))?
                .parse()
                .unwrap_or(0),
            status: CourseStatus::from_str(status_str).unwrap_or(CourseStatus::Draft),
            created_at: DateTime::parse_from_rfc3339(created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn to_data_map(course: &Course) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), course.id.clone());
        data.insert("name".to_string(), course.name.clone());
        data.insert(
            "description".to_string(),
            course.description.clone().unwrap_or_default(),
        );
        data.insert("code".to_string(), course.code.clone());
        data.insert("credits".to_string(), course.credits.to_string());
        data.insert("status".to_string(), course.status.as_str().to_string());
        data.insert("created_at".to_string(), course.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), course.updated_at.to_rfc3339());
        data.insert("price".to_string(), course.price.to_string());
        data.insert("duration".to_string(), course.duration.to_string());
        data
    }

    async fn get_user_id(&self) -> Result<String, DomainError> {
        let session = self.session.lock().await;
        session
            .user_id
            .clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))
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
impl CourseRepository for MemoryBackedCourseRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts/updates first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "courses", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Ok(Some(Self::course_from_data(data)?));
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "courses", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Ok(Some(Self::course_from_data(data)?));
                }
            }
            if buf.has_pending_delete(&user_id, "courses", id) {
                return Ok(None);
            }
        }

        // Check entity cache or query Turso
        let row_data: Option<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_entity(&user_id, "courses", id) {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql = "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration
                           FROM courses WHERE id = ?1";
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
                buf.set_cached_entity(&user_id, "courses", id, data.clone());
                data
            }
        };

        match row_data {
            Some(data) => Ok(Some(Self::course_from_data(&data)?)),
            None => Ok(None),
        }
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts
        {
            let buf = self.memory_buffer.lock().await;
            let pending_inserts = buf.scan_pending_inserts(&user_id, "courses");
            for op in pending_inserts.iter().rev() {
                if let BufferedOperation::Insert { data, .. } = op {
                    if data.get("code").map(|v| v.as_str()) == Some(code) {
                        return Ok(Some(Self::course_from_data(data)?));
                    }
                }
            }
            let pending_updates = buf.scan_pending_updates(&user_id, "courses");
            for op in pending_updates.iter().rev() {
                if let BufferedOperation::Update { data, .. } = op {
                    if data.get("code").map(|v| v.as_str()) == Some(code) {
                        return Ok(Some(Self::course_from_data(data)?));
                    }
                }
            }
        }

        // Read from Turso
        let sql = "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration
                   FROM courses WHERE code = ?1";
        let mut rows = self
            .query_turso(&user_id, sql, libsql::params![code])
            .await?;
        match rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            Some(row) => Ok(Some(Self::row_to_course(&row)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, course: &Course) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;
        let data = Self::to_data_map(course);
        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "courses".to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn update(&self, course: &Course) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut data = HashMap::new();
        data.insert("name".to_string(), course.name.clone());
        data.insert(
            "description".to_string(),
            course.description.clone().unwrap_or_default(),
        );
        data.insert("code".to_string(), course.code.clone());
        data.insert("credits".to_string(), course.credits.to_string());
        data.insert("status".to_string(), course.status.as_str().to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("price".to_string(), course.price.to_string());
        data.insert("duration".to_string(), course.duration.to_string());
        data.insert("id".to_string(), course.id.clone()); // for deserialization

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "courses".to_string(),
                id: course.id.clone(),
                data,
            },
        );
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        // Soft delete — set status to archived
        let mut data = HashMap::new();
        data.insert("status".to_string(), "archived".to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("id".to_string(), id.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "courses".to_string(),
                id: id.to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Course>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Step 1: Check cache or query Turso
        let base_rows: Vec<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_list(&user_id, "courses") {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql = "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration
                           FROM courses WHERE status != 'archived' ORDER BY name";
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
                buf.set_cached_list(&user_id, "courses", raw_rows.clone());
                raw_rows
            }
        };

        // Step 2: Convert rows to domain entities
        let mut results: Vec<Course> = base_rows
            .iter()
            .map(|data| Self::course_from_data(data))
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "courses");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                let course = Self::course_from_data(data)?;
                if course.status.as_str() != "archived" {
                    results.push(course);
                }
            }
        }

        let pending_updates = buf.scan_pending_updates(&user_id, "courses");
        for op in &pending_updates {
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
                let updated = Self::course_from_data(data)?;
                if let Some(pos) = results.iter().position(|c| c.id == *update_id) {
                    if updated.status.as_str() != "archived" {
                        results[pos] = updated;
                    } else {
                        results.remove(pos);
                    }
                } else if updated.status.as_str() != "archived" {
                    results.push(updated);
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "courses");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|c| c.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn find_all_archived(&self) -> Result<Vec<Course>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Read from Turso
        let sql = "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration
                   FROM courses WHERE status = 'archived' ORDER BY name";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

        let mut results: Vec<Course> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            results.push(Self::row_to_course(&row)?);
        }

        // Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "courses");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                let course = Self::course_from_data(data)?;
                if course.status.as_str() == "archived" {
                    results.push(course);
                }
            }
        }

        let pending_updates = buf.scan_pending_updates(&user_id, "courses");
        for op in &pending_updates {
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
                let updated = Self::course_from_data(data)?;
                if let Some(pos) = results.iter().position(|c| c.id == *update_id) {
                    if updated.status.as_str() == "archived" {
                        results[pos] = updated;
                    } else {
                        results.remove(pos);
                    }
                } else if updated.status.as_str() == "archived" {
                    results.push(updated);
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "courses");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|c| c.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn restore(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut data = HashMap::new();
        data.insert("status".to_string(), "draft".to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("id".to_string(), id.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "courses".to_string(),
                id: id.to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn hard_delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Delete {
                table: "courses".to_string(),
                id: id.to_string(),
            },
        );
        Ok(())
    }
}
