//! MemoryBuffer-backed Attendance Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::AttendanceRepository;
use crate::domain::entities::attendance::{Attendance, AttendanceStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedAttendanceRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedAttendanceRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(attendance: &Attendance) -> CachedEntity {
        CachedEntity {
            id: attendance.id.clone(),
            data: HashMap::from([
                ("id".to_string(), attendance.id.clone()),
                ("student_id".to_string(), attendance.student_id.clone()),
                ("group_id".to_string(), attendance.group_id.clone()),
                ("date".to_string(), attendance.date.to_rfc3339()),
                ("status".to_string(), attendance.status.as_str().to_string()),
                ("notes".to_string(), attendance.notes.clone().unwrap_or_default()),
                ("created_at".to_string(), attendance.created_at.to_rfc3339()),
                ("updated_at".to_string(), attendance.updated_at.to_rfc3339()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<Attendance> {
        Some(Attendance {
            id: cached.data.get("id")?.clone(),
            student_id: cached.data.get("student_id")?.clone(),
            group_id: cached.data.get("group_id")?.clone(),
            date: cached.data.get("date")
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(s).ok()
                        .map(|dt| dt.with_timezone(&Utc))
                        .or_else(|| {
                            NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                                .ok()
                                .map(|ndt| Utc.from_utc_datetime(&ndt))
                        })
                })
                .unwrap_or_else(|| Utc::now()),
            status: {
                let s = cached.data.get("status")?;
                AttendanceStatus::from_str(s).unwrap_or(AttendanceStatus::Present)
            },
            notes: Self::opt_string(cached, "notes"),
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            updated_at: Utc::now(),
        })
    }

    fn opt_string(cached: &CachedEntity, key: &str) -> Option<String> {
        cached.data.get(key).and_then(|v| {
            if v.is_empty() { None } else { Some(v.clone()) }
        })
    }

    fn row_to_attendance(row: &rusqlite::Row<'_>) -> rusqlite::Result<Attendance> {
        let status_str: String = row.get(4)?;
        let date_str: String = row.get(3)?;
        let created_str: String = row.get(6)?;

        let date = DateTime::parse_from_rfc3339(&date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| Utc.from_utc_datetime(&ndt))
            })
            .unwrap_or_else(|_| Utc::now());

        Ok(Attendance {
            id: row.get(0)?,
            student_id: row.get(1)?,
            group_id: row.get(2)?,
            date,
            status: AttendanceStatus::from_str(&status_str).unwrap_or(AttendanceStatus::Present),
            notes: row.get(5)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: Utc::now(),
        })
    }
}

impl AttendanceRepository for MemoryBackedAttendanceRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Attendance>, DomainError> {
        let cache_key = format!("attendance:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_attendance) {
            Ok(attendance) => Ok(Some(attendance)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(attendance).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "attendance".to_string(),
            data,
        });
        Ok(())
    }

    fn update(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(attendance).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "attendance".to_string(),
            id: attendance.id.clone(),
            data,
        });
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "attendance".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Attendance>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance ORDER BY date DESC";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_attendance)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Attendance>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE student_id = ? ORDER BY date DESC";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![student_id], Self::row_to_attendance)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_group_and_date(
        &self,
        group_id: &str,
        date: DateTime<Utc>,
    ) -> Result<Vec<Attendance>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let date_str = date.format("%Y-%m-%d").to_string();
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE group_id = ? AND date LIKE ? ORDER BY date DESC";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![group_id, format!("{}%", date_str)], Self::row_to_attendance)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn count_absences_by_student_and_group(
        &self,
        student_id: &str,
        group_id: &str,
    ) -> Result<i32, DomainError> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT COUNT(*) FROM attendance
                   WHERE student_id = ? AND group_id = ? AND status = 'absent'";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let count: i32 = conn
            .query_row(sql, rusqlite::params![student_id, group_id], |row| row.get(0))
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(count)
    }

    fn count_absences_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<(String, i32)>, DomainError> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT student_id, COUNT(*) as absence_count FROM attendance
                   WHERE group_id = ? AND status = 'absent'
                   GROUP BY student_id";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![group_id], |row| {
                let student_id: String = row.get(0)?;
                let count: i32 = row.get(1)?;
                Ok((student_id, count))
            })
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }
}
