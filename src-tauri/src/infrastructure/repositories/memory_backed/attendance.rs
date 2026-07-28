//! MemoryBacked Attendance Repository
//!
//! Implements AttendanceRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5c: Complex repositories.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use crate::application::ports::AttendanceRepository;
use crate::domain::entities::attendance::{Attendance, AttendanceStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct MemoryBackedAttendanceRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedAttendanceRepository {
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

    fn attendance_to_data(attendance: &Attendance) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), attendance.id.clone());
        data.insert("student_id".to_string(), attendance.student_id.clone());
        data.insert("group_id".to_string(), attendance.group_id.clone());
        data.insert("date".to_string(), attendance.date.to_rfc3339());
        data.insert("status".to_string(), attendance.status.as_str().to_string());
        if let Some(ref notes) = attendance.notes {
            data.insert("notes".to_string(), notes.clone());
        }
        data.insert("created_at".to_string(), attendance.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), attendance.updated_at.to_rfc3339());
        data
    }

    fn attendance_from_data(data: &HashMap<String, String>) -> Result<Attendance, String> {
        let id = data.get("id").ok_or_else(|| "missing id".to_string())?.clone();
        let student_id = data.get("student_id").ok_or_else(|| "missing student_id".to_string())?.clone();
        let group_id = data.get("group_id").ok_or_else(|| "missing group_id".to_string())?.clone();
        let date = data.get("date").ok_or_else(|| "missing date".to_string())?
            .parse::<DateTime<Utc>>().map_err(|e| e.to_string())?;
        let status_str = data.get("status").ok_or_else(|| "missing status".to_string())?;
        let status = AttendanceStatus::from_str(status_str)
            .ok_or_else(|| format!("invalid status: {}", status_str))?;
        let notes = data.get("notes").cloned();
        let created_at = data.get("created_at").ok_or_else(|| "missing created_at".to_string())?
            .parse::<DateTime<Utc>>().map_err(|e| e.to_string())?;
        let updated_at = data.get("updated_at").ok_or_else(|| "missing updated_at".to_string())?
            .parse::<DateTime<Utc>>().map_err(|e| e.to_string())?;

        Ok(Attendance {
            id, student_id, group_id, date, status, notes, created_at, updated_at,
        })
    }
}

#[async_trait]
impl AttendanceRepository for MemoryBackedAttendanceRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Attendance>, DomainError> {
        let user_id = self.session.lock().await.user_id.clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?;

        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "attendance", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Self::attendance_from_data(data).map_err(|e| DomainError::Database(e)).map(Some);
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "attendance", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Self::attendance_from_data(data).map_err(|e| DomainError::Database(e)).map(Some);
                }
            }
            if buf.has_pending_delete(&user_id, "attendance", id) {
                return Ok(None);
            }
        }

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id)
            .ok_or_else(|| DomainError::Database("No connection".to_string()))?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at, updated_at FROM attendance WHERE id = ?1";
        let mut rows = conn.query(sql, libsql::params![id]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Self::row_to_attendance(&row).map(Some),
            None => Ok(None),
        }
    }

    async fn save(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let user_id = self.session.lock().await.user_id.clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "attendance".to_string(),
                data: Self::attendance_to_data(attendance),
            },
        );
        Ok(())
    }

    async fn update(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let user_id = self.session.lock().await.user_id.clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "attendance".to_string(),
                id: attendance.id.clone(),
                data: Self::attendance_to_data(attendance),
            },
        );
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.session.lock().await.user_id.clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Delete {
                table: "attendance".to_string(),
                id: id.to_string(),
            },
        );
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Attendance>, DomainError> {
        let user_id = self.session.lock().await.user_id.clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?;

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id)
            .ok_or_else(|| DomainError::Database("No connection".to_string()))?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at, updated_at FROM attendance ORDER BY date DESC";
        let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results: Vec<Attendance> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_attendance(&row)?);
        }

        let buf = self.memory_buffer.lock().await;
        let pending_data: Vec<HashMap<String, String>> = buf
            .scan_pending_inserts(&user_id, "attendance")
            .into_iter()
            .filter_map(|op| {
                if let BufferedOperation::Insert { data, .. } = op {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .collect();
        drop(buf);
        for data in pending_data {
            if let Ok(attendance) = Self::attendance_from_data(&data) {
                results.push(attendance);
            }
        }

        Ok(results)
    }

    async fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Attendance>, DomainError> {
        let user_id = self.session.lock().await.user_id.clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?;

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id)
            .ok_or_else(|| DomainError::Database("No connection".to_string()))?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at, updated_at FROM attendance WHERE student_id = ?1 ORDER BY date DESC";
        let mut rows = conn.query(sql, libsql::params![student_id]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results: Vec<Attendance> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_attendance(&row)?);
        }

        let buf = self.memory_buffer.lock().await;
        let pending_data: Vec<HashMap<String, String>> = buf
            .scan_pending_inserts(&user_id, "attendance")
            .into_iter()
            .filter_map(|op| {
                if let BufferedOperation::Insert { data, .. } = op {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .collect();
        drop(buf);
        for data in pending_data {
            if let Ok(attendance) = Self::attendance_from_data(&data) {
                if attendance.student_id == student_id {
                    results.push(attendance);
                }
            }
        }

        Ok(results)
    }

    async fn find_by_group_and_date(&self, group_id: &str, date: DateTime<Utc>) -> Result<Vec<Attendance>, DomainError> {
        let user_id = self.session.lock().await.user_id.clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))?;
        let date_str = date.to_rfc3339();

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id)
            .ok_or_else(|| DomainError::Database("No connection".to_string()))?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at, updated_at FROM attendance WHERE group_id = ?1 AND date = ?2";
        let mut rows = conn.query(sql, libsql::params![group_id, date_str]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results: Vec<Attendance> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_attendance(&row)?);
        }

        let buf = self.memory_buffer.lock().await;
        let pending_data: Vec<HashMap<String, String>> = buf
            .scan_pending_inserts(&user_id, "attendance")
            .into_iter()
            .filter_map(|op| {
                if let BufferedOperation::Insert { data, .. } = op {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .collect();
        drop(buf);
        for data in pending_data {
            if let Ok(attendance) = Self::attendance_from_data(&data) {
                if attendance.group_id == group_id && attendance.date == date {
                    results.push(attendance);
                }
            }
        }

        Ok(results)
    }

    async fn count_absences_by_student_and_group(&self, student_id: &str, group_id: &str) -> Result<i32, DomainError> {
        let attendances = self.find_by_student_id(student_id).await?;
        let count = attendances.iter()
            .filter(|a| a.group_id == group_id && a.status == AttendanceStatus::Absent)
            .count() as i32;
        Ok(count)
    }

    async fn count_absences_by_group(&self, group_id: &str) -> Result<Vec<(String, i32)>, DomainError> {
        let all = self.find_all().await?;
        let mut counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for a in all {
            if a.group_id == group_id && a.status == AttendanceStatus::Absent {
                *counts.entry(a.student_id.clone()).or_insert(0) += 1;
            }
        }
        Ok(counts.into_iter().collect())
    }
}

impl MemoryBackedAttendanceRepository {
    fn row_to_attendance(row: &libsql::Row) -> Result<Attendance, DomainError> {
        let id: String = row.get(0).map_err(|e| DomainError::Database(e.to_string()))?;
        let student_id: String = row.get(1).map_err(|e| DomainError::Database(e.to_string()))?;
        let group_id: String = row.get(2).map_err(|e| DomainError::Database(e.to_string()))?;
        let date_str: String = row.get(3).map_err(|e| DomainError::Database(e.to_string()))?;
        let date = date_str.parse::<DateTime<Utc>>()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let status_str: String = row.get(4).map_err(|e| DomainError::Database(e.to_string()))?;
        let status = AttendanceStatus::from_str(&status_str).ok_or_else(|| {
            DomainError::Validation(format!("invalid attendance status: {}", status_str))
        })?;
        let notes: Option<String> = row.get(5).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(6).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_at = created_str.parse::<DateTime<Utc>>()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row.get(7).map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_at = updated_str.parse::<DateTime<Utc>>()
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(Attendance {
            id, student_id, group_id, date, status, notes, created_at, updated_at,
        })
    }
}
