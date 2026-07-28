//! MemoryBacked Student Repository
//!
//! Implements StudentRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5b: 6 MemoryBacked repositories.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::StudentRepository;
use crate::domain::entities::Student;
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

/// MemoryBuffer-backed implementation of StudentRepository.
pub struct MemoryBackedStudentRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedStudentRepository {
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

    fn row_to_student(row: &libsql::Row) -> Result<Student, DomainError> {
        let birth_date_str: Option<String> = row.get(8).map_err(|e| DomainError::Database(e.to_string()))?;
        let guardian_name: Option<String> = row.get(9).map_err(|e| DomainError::Database(e.to_string()))?;
        let guardian_document: Option<String> = row.get(10).map_err(|e| DomainError::Database(e.to_string()))?;
        let guardian_phone: Option<String> = row.get(11).map_err(|e| DomainError::Database(e.to_string()))?;
        let active: i32 = row.get(12).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(13).map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row.get(14).map_err(|e| DomainError::Database(e.to_string()))?;
        let course_id: Option<String> = row.get(15).map_err(|e| DomainError::Database(e.to_string()))?;
        let group_id: Option<String> = row.get(16).map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(Student {
            id: row.get(0).map_err(|e| DomainError::Database(e.to_string()))?,
            user_id: row.get(1).map_err(|e| DomainError::Database(e.to_string()))?,
            first_name: row.get(2).map_err(|e| DomainError::Database(e.to_string()))?,
            last_name: row.get(3).map_err(|e| DomainError::Database(e.to_string()))?,
            document_type: row.get(4).map_err(|e| DomainError::Database(e.to_string()))?,
            document_number: row.get(5).map_err(|e| DomainError::Database(e.to_string()))?,
            email: row.get(6).map_err(|e| DomainError::Database(e.to_string()))?,
            phone: row.get(7).map_err(|e| DomainError::Database(e.to_string()))?,
            address: row.get::<Option<String>>(17).map_err(|e| DomainError::Database(e.to_string()))?,
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

    fn student_from_data(data: &HashMap<String, String>) -> Result<Student, DomainError> {
        let active: i32 = data
            .get("active")
            .ok_or_else(|| DomainError::Database("missing active".into()))?
            .parse()
            .unwrap_or(0);
        let created_str = data.get("created_at").ok_or_else(|| DomainError::Database("missing created_at".into()))?;
        let updated_str = data.get("updated_at").ok_or_else(|| DomainError::Database("missing updated_at".into()))?;

        let birth_date_str: Option<&String> = data.get("birth_date");
        let birth_date = birth_date_str.and_then(|s| {
            if s.is_empty() { return None; }
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|| {
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .ok()
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                })
        });

        Ok(Student {
            id: data.get("id").ok_or_else(|| DomainError::Database("missing id".into()))?.clone(),
            user_id: data.get("user_id").ok_or_else(|| DomainError::Database("missing user_id".into()))?.clone(),
            first_name: data.get("first_name").ok_or_else(|| DomainError::Database("missing first_name".into()))?.clone(),
            last_name: data.get("last_name").ok_or_else(|| DomainError::Database("missing last_name".into()))?.clone(),
            document_type: data.get("document_type").ok_or_else(|| DomainError::Database("missing document_type".into()))?.clone(),
            document_number: data.get("document_number").ok_or_else(|| DomainError::Database("missing document_number".into()))?.clone(),
            email: data.get("email").ok_or_else(|| DomainError::Database("missing email".into()))?.clone(),
            phone: data.get("phone").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            address: data.get("address").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            birth_date,
            guardian_name: data.get("guardian_name").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            guardian_document: data.get("guardian_document").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            guardian_phone: data.get("guardian_phone").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            course_id: data.get("course_id").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            group_id: data.get("group_id").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            active: active != 0,
            created_at: DateTime::parse_from_rfc3339(created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn to_data_map(student: &Student) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), student.id.clone());
        data.insert("user_id".to_string(), student.user_id.clone());
        data.insert("first_name".to_string(), student.first_name.clone());
        data.insert("last_name".to_string(), student.last_name.clone());
        data.insert("document_type".to_string(), student.document_type.clone());
        data.insert("document_number".to_string(), student.document_number.clone());
        data.insert("email".to_string(), student.email.clone());
        data.insert("phone".to_string(), student.phone.clone().unwrap_or_default());
        data.insert("address".to_string(), student.address.clone().unwrap_or_default());
        data.insert("birth_date".to_string(), student.birth_date.map(|d| d.to_rfc3339()).unwrap_or_default());
        data.insert("guardian_name".to_string(), student.guardian_name.clone().unwrap_or_default());
        data.insert("guardian_document".to_string(), student.guardian_document.clone().unwrap_or_default());
        data.insert("guardian_phone".to_string(), student.guardian_phone.clone().unwrap_or_default());
        data.insert("active".to_string(), if student.active { "1".to_string() } else { "0".to_string() });
        data.insert("created_at".to_string(), student.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), student.updated_at.to_rfc3339());
        data.insert("course_id".to_string(), student.course_id.clone().unwrap_or_default());
        data.insert("group_id".to_string(), student.group_id.clone().unwrap_or_default());
        data
    }

    async fn get_user_id(&self) -> Result<String, DomainError> {
        let session = self.session.lock().await;
        session
            .user_id
            .clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))
    }

    async fn query_turso(&self, user_id: &str, sql: &str, params: impl libsql::params::IntoParams) -> Result<libsql::Rows, DomainError> {
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
impl StudentRepository for MemoryBackedStudentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Student>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts/updates first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "students", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Ok(Some(Self::student_from_data(data)?));
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "students", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Ok(Some(Self::student_from_data(data)?));
                }
            }
            if buf.has_pending_delete(&user_id, "students", id) {
                return Ok(None);
            }
        }

        // Read from Turso
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE id = ?1";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![id]).await?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_student(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, target_user_id: &str) -> Result<Option<Student>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts
        {
            let buf = self.memory_buffer.lock().await;
            let pending_inserts = buf.scan_pending_inserts(&user_id, "students");
            for op in pending_inserts.iter().rev() {
                if let BufferedOperation::Insert { data, .. } = op {
                    if data.get("user_id").map(|v| v.as_str()) == Some(target_user_id) {
                        return Ok(Some(Self::student_from_data(data)?));
                    }
                }
            }
            let pending_updates = buf.scan_pending_updates(&user_id, "students");
            for op in pending_updates.iter().rev() {
                if let BufferedOperation::Update { data, .. } = op {
                    if data.get("user_id").map(|v| v.as_str()) == Some(target_user_id) {
                        return Ok(Some(Self::student_from_data(data)?));
                    }
                }
            }
        }

        // Read from Turso
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE user_id = ?1";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![target_user_id]).await?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_student(&row)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, student: &Student) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;
        let data = Self::to_data_map(student);
        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Insert {
            table: "students".to_string(),
            data,
        });
        Ok(())
    }

    async fn update(&self, student: &Student) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut data = HashMap::new();
        data.insert("first_name".to_string(), student.first_name.clone());
        data.insert("last_name".to_string(), student.last_name.clone());
        data.insert("email".to_string(), student.email.clone());
        data.insert("phone".to_string(), student.phone.clone().unwrap_or_default());
        data.insert("address".to_string(), student.address.clone().unwrap_or_default());
        data.insert("birth_date".to_string(), student.birth_date.map(|d| d.to_rfc3339()).unwrap_or_default());
        data.insert("guardian_name".to_string(), student.guardian_name.clone().unwrap_or_default());
        data.insert("guardian_document".to_string(), student.guardian_document.clone().unwrap_or_default());
        data.insert("guardian_phone".to_string(), student.guardian_phone.clone().unwrap_or_default());
        data.insert("active".to_string(), if student.active { "1".to_string() } else { "0".to_string() });
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("course_id".to_string(), student.course_id.clone().unwrap_or_default());
        data.insert("group_id".to_string(), student.group_id.clone().unwrap_or_default());
        data.insert("id".to_string(), student.id.clone()); // for deserialization

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Update {
            table: "students".to_string(),
            id: student.id.clone(),
            data,
        });
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        // Soft delete — set active = 0
        let mut data = HashMap::new();
        data.insert("active".to_string(), "0".to_string());
        data.insert("updated_at".to_string(), Utc::now().to_rfc3339());
        data.insert("id".to_string(), id.to_string());

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Update {
            table: "students".to_string(),
            id: id.to_string(),
            data,
        });
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Student>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Read from Turso
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE active = 1 ORDER BY last_name, first_name";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

        let mut results: Vec<Student> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_student(&row)?);
        }

        // Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        // Add pending inserts
        let pending_inserts = buf.scan_pending_inserts(&user_id, "students");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                let student = Self::student_from_data(data)?;
                if student.active {
                    results.push(student);
                }
            }
        }

        // Apply pending updates
        let pending_updates = buf.scan_pending_updates(&user_id, "students");
        for op in &pending_updates {
            if let BufferedOperation::Update { id: update_id, data, .. } = op {
                let updated = Self::student_from_data(data)?;
                if let Some(pos) = results.iter().position(|s| s.id == *update_id) {
                    if updated.active {
                        results[pos] = updated;
                    } else {
                        results.remove(pos);
                    }
                } else if updated.active {
                    results.push(updated);
                }
            }
        }

        // Remove pending deletes
        let pending_deletes = buf.scan_pending_deletes(&user_id, "students");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|s| s.id != *del_id);
            }
        }

        Ok(results)
    }
}
