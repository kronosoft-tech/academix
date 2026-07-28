//! MemoryBuffer-backed Course Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::CourseRepository;
use crate::domain::entities::course::{Course, CourseStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedCourseRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedCourseRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(course: &Course) -> CachedEntity {
        CachedEntity {
            id: course.id.clone(),
            data: HashMap::from([
                ("id".to_string(), course.id.clone()),
                ("name".to_string(), course.name.clone()),
                ("description".to_string(), course.description.clone().unwrap_or_default()),
                ("code".to_string(), course.code.clone()),
                ("credits".to_string(), course.credits.to_string()),
                ("price".to_string(), course.price.to_string()),
                ("duration".to_string(), course.duration.to_string()),
                ("status".to_string(), course.status.as_str().to_string()),
                ("created_at".to_string(), course.created_at.to_rfc3339()),
                ("updated_at".to_string(), course.updated_at.to_rfc3339()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<Course> {
        Some(Course {
            id: cached.data.get("id")?.clone(),
            name: cached.data.get("name")?.clone(),
            description: {
                let v = cached.data.get("description")?;
                if v.is_empty() { None } else { Some(v.clone()) }
            },
            code: cached.data.get("code")?.clone(),
            credits: cached.data.get("credits")?.parse().unwrap_or(0),
            price: cached.data.get("price")?.parse().unwrap_or(200000.0),
            duration: cached.data.get("duration")?.parse().unwrap_or(0),
            status: {
                let s = cached.data.get("status")?;
                CourseStatus::from_str(s).unwrap_or(CourseStatus::Draft)
            },
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            updated_at: cached.data.get("updated_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
        })
    }

    fn row_to_course(row: &rusqlite::Row<'_>) -> rusqlite::Result<Course> {
        let status_str: String = row.get(5)?;
        let created_str: String = row.get(6)?;
        let updated_str: String = row.get(7)?;
        let price: f64 = row.get(8).unwrap_or(200000.0);
        let duration: i32 = row.get::<_, Option<i32>>(9)?.unwrap_or(0);

        Ok(Course {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            code: row.get(3)?,
            credits: row.get(4)?,
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
}

impl CourseRepository for MemoryBackedCourseRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError> {
        let cache_key = format!("course:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                    FROM courses WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_course) {
            Ok(course) => Ok(Some(course)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError> {
        let cache_key = format!("course:code:{}", code);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                    FROM courses WHERE code = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [code], Self::row_to_course) {
            Ok(course) => Ok(Some(course)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, course: &Course) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(course).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "courses".to_string(),
            data,
        });
        Ok(())
    }

    fn update(&self, course: &Course) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(course).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "courses".to_string(),
            id: course.id.clone(),
            data,
        });
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "courses".to_string(),
            id: id.to_string(),
            data: HashMap::from([
                ("status".to_string(), "archived".to_string()),
            ]),
        });
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Course>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                    FROM courses WHERE status != 'archived' ORDER BY name";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_all_archived(&self) -> Result<Vec<Course>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                    FROM courses WHERE status = 'archived' ORDER BY name";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn restore(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "courses".to_string(),
            id: id.to_string(),
            data: HashMap::from([
                ("status".to_string(), "draft".to_string()),
            ]),
        });
        Ok(())
    }

    fn hard_delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "courses".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }
}
