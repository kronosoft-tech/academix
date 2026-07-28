//! MemoryBuffer-backed Student Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::StudentRepository;
use crate::domain::entities::Student;
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedStudentRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedStudentRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(student: &Student) -> CachedEntity {
        CachedEntity {
            id: student.id.clone(),
            data: HashMap::from([
                ("id".to_string(), student.id.clone()),
                ("user_id".to_string(), student.user_id.clone()),
                ("first_name".to_string(), student.first_name.clone()),
                ("last_name".to_string(), student.last_name.clone()),
                ("document_type".to_string(), student.document_type.clone()),
                ("document_number".to_string(), student.document_number.clone()),
                ("email".to_string(), student.email.clone()),
                ("phone".to_string(), student.phone.clone().unwrap_or_default()),
                ("address".to_string(), student.address.clone().unwrap_or_default()),
                ("birth_date".to_string(), student.birth_date.map(|dt| dt.to_rfc3339()).unwrap_or_default()),
                ("guardian_name".to_string(), student.guardian_name.clone().unwrap_or_default()),
                ("guardian_document".to_string(), student.guardian_document.clone().unwrap_or_default()),
                ("guardian_phone".to_string(), student.guardian_phone.clone().unwrap_or_default()),
                ("course_id".to_string(), student.course_id.clone().unwrap_or_default()),
                ("group_id".to_string(), student.group_id.clone().unwrap_or_default()),
                ("active".to_string(), if student.active { "1" } else { "0" }.to_string()),
                ("created_at".to_string(), student.created_at.to_rfc3339()),
                ("updated_at".to_string(), student.updated_at.to_rfc3339()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<Student> {
        let birth_date_str = cached.data.get("birth_date")?;
        let birth_date = if birth_date_str.is_empty() {
            None
        } else {
            DateTime::parse_from_rfc3339(birth_date_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|| {
                    chrono::NaiveDate::parse_from_str(birth_date_str, "%Y-%m-%d")
                        .ok()
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                })
        };

        Some(Student {
            id: cached.data.get("id")?.clone(),
            user_id: cached.data.get("user_id")?.clone(),
            first_name: cached.data.get("first_name")?.clone(),
            last_name: cached.data.get("last_name")?.clone(),
            document_type: cached.data.get("document_type")?.clone(),
            document_number: cached.data.get("document_number")?.clone(),
            email: cached.data.get("email")?.clone(),
            phone: Self::opt_string(cached, "phone"),
            address: Self::opt_string(cached, "address"),
            birth_date,
            guardian_name: Self::opt_string(cached, "guardian_name"),
            guardian_document: Self::opt_string(cached, "guardian_document"),
            guardian_phone: Self::opt_string(cached, "guardian_phone"),
            course_id: Self::opt_string(cached, "course_id"),
            group_id: Self::opt_string(cached, "group_id"),
            active: cached.data.get("active")? == "1",
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

    fn opt_string(cached: &CachedEntity, key: &str) -> Option<String> {
        cached.data.get(key).and_then(|v| {
            if v.is_empty() { None } else { Some(v.clone()) }
        })
    }

    fn row_to_student(row: &rusqlite::Row<'_>) -> rusqlite::Result<Student> {
        let birth_date_str: Option<String> = row.get(8)?;
        let guardian_name: Option<String> = row.get(9)?;
        let guardian_document: Option<String> = row.get(10)?;
        let guardian_phone: Option<String> = row.get(11)?;
        let active: i32 = row.get(12)?;
        let created_str: String = row.get(13)?;
        let updated_str: String = row.get(14)?;
        let course_id: Option<String> = row.get(15)?;
        let group_id: Option<String> = row.get(16)?;

        Ok(Student {
            id: row.get(0)?,
            user_id: row.get(1)?,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
            document_type: row.get(4)?,
            document_number: row.get(5)?,
            email: row.get(6)?,
            phone: row.get(7)?,
            address: row.get::<_, Option<String>>(17)?,
            birth_date: birth_date_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                            .ok()
                            .and_then(|d| d.and_hms_opt(0, 0, 0))
                            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                    })
            }),
            guardian_name,
            guardian_document,
            guardian_phone,
            course_id,
            group_id,
            active: active != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl StudentRepository for MemoryBackedStudentRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Student>, DomainError> {
        let cache_key = format!("student:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_student) {
            Ok(student) => Ok(Some(student)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_user_id(&self, user_id: &str) -> Result<Option<Student>, DomainError> {
        let cache_key = format!("student:user:{}", user_id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE user_id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [user_id], Self::row_to_student) {
            Ok(student) => Ok(Some(student)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, student: &Student) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(student).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "students".to_string(),
            data,
        });
        Ok(())
    }

    fn update(&self, student: &Student) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(student).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "students".to_string(),
            id: student.id.clone(),
            data,
        });
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "students".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Student>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE active = 1 ORDER BY last_name, first_name";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_student)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }
}
