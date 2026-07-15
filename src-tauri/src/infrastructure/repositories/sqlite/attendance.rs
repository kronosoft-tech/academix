//! Attendance SQLite Repository
//!
//! Implements AttendanceRepository using SQLite.

use crate::application::ports::AttendanceRepository;
use crate::domain::entities::attendance::{Attendance, AttendanceStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::sync::Arc;

/// SQLite implementation of AttendanceRepository
#[derive(Clone)]
pub struct SqliteAttendanceRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteAttendanceRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_attendance(row: &rusqlite::Row<'_>) -> rusqlite::Result<Attendance> {
        let status_str: String = row.get(4)?;
        let date_str: String = row.get(3)?;
        let created_str: String = row.get(6)?;

        // Parse date - may be just date or full datetime
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

impl AttendanceRepository for SqliteAttendanceRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Attendance>, DomainError> {
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_attendance)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn save(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let sql =
            "INSERT INTO attendance (id, student_id, group_id, date, status, notes, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &attendance.id,
                    &attendance.student_id,
                    &attendance.group_id,
                    &attendance.date.to_rfc3339(),
                    &attendance.status.as_str().to_string(),
                    &attendance.notes,
                    &attendance.created_at.to_rfc3339(),
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn update(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let sql = "UPDATE attendance 
                   SET student_id = ?, group_id = ?, date = ?, status = ?, notes = ?
                   WHERE id = ?";

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &attendance.student_id,
                    &attendance.group_id,
                    &attendance.date.to_rfc3339(),
                    &attendance.status.as_str().to_string(),
                    &attendance.notes,
                    &attendance.id,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Attendance", &attendance.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM attendance WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Attendance", id));
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Attendance>, DomainError> {
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance ORDER BY date DESC";

        self.pool
            .query(sql, &[], Self::row_to_attendance)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Attendance>, DomainError> {
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE student_id = ? ORDER BY date DESC";

        self.pool
            .query(sql, &[&student_id], Self::row_to_attendance)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_group_and_date(
        &self,
        group_id: &str,
        date: DateTime<Utc>,
    ) -> Result<Vec<Attendance>, DomainError> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE group_id = ? AND date LIKE ? ORDER BY date DESC";

        self.pool
            .query(
                sql,
                &[&group_id, &format!("{}%", date_str)],
                Self::row_to_attendance,
            )
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn count_absences_by_student_and_group(
        &self,
        student_id: &str,
        group_id: &str,
    ) -> Result<i32, DomainError> {
        let sql = "SELECT COUNT(*) FROM attendance
                   WHERE student_id = ? AND group_id = ? AND status = 'absent'";

        self.pool
            .query_row(sql, &[&student_id, &group_id], |row| row.get::<_, i32>(0))
            .map_err(|e| DomainError::Validation(e.to_string()))?
            .ok_or_else(|| DomainError::Validation("Count query returned no result".to_string()))
    }

    fn count_absences_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<(String, i32)>, DomainError> {
        let sql = "SELECT student_id, COUNT(*) as absence_count FROM attendance
                   WHERE group_id = ? AND status = 'absent'
                   GROUP BY student_id";

        self.pool
            .query(sql, &[&group_id], |row| {
                let student_id: String = row.get(0)?;
                let count: i32 = row.get(1)?;
                Ok((student_id, count))
            })
            .map_err(|e| DomainError::Validation(e.to_string()))
    }
}
