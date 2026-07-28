use async_trait::async_trait;
use crate::application::ports::AttendanceRepository;
use crate::domain::entities::attendance::{Attendance, AttendanceStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::local_db;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

#[derive(Clone)]
pub struct SqliteAttendanceRepository;

impl SqliteAttendanceRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_attendance(row: &libsql::Row) -> Result<Attendance, DomainError> {
        let status_str: String = row.get(4).map_err(|e| DomainError::Database(e.to_string()))?;
        let date_str: String = row.get(3).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(6).map_err(|e| DomainError::Database(e.to_string()))?;

        let date = DateTime::parse_from_rfc3339(&date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| Utc.from_utc_datetime(&ndt))
            })
            .unwrap_or_else(|_| Utc::now());

        Ok(Attendance {
            id: row.get(0).map_err(|e| DomainError::Database(e.to_string()))?,
            student_id: row.get(1).map_err(|e| DomainError::Database(e.to_string()))?,
            group_id: row.get(2).map_err(|e| DomainError::Database(e.to_string()))?,
            date,
            status: AttendanceStatus::from_str(&status_str).unwrap_or(AttendanceStatus::Present),
            notes: row.get(5).map_err(|e| DomainError::Database(e.to_string()))?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: Utc::now(),
        })
    }
}

impl Default for SqliteAttendanceRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AttendanceRepository for SqliteAttendanceRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Attendance>, DomainError> {
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![id]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => {
                let attendance = Self::row_to_attendance(&row)?;
                Ok(Some(attendance))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let sql =
            "INSERT INTO attendance (id, student_id, group_id, date, status, notes, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(
            sql,
            libsql::params![
                attendance.id.clone(),
                attendance.student_id.clone(),
                attendance.group_id.clone(),
                attendance.date.to_rfc3339(),
                attendance.status.as_str(),
                attendance.notes.clone(),
                attendance.created_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let sql = "UPDATE attendance 
                   SET student_id = ?1, group_id = ?2, date = ?3, status = ?4, notes = ?5
                   WHERE id = ?6";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    attendance.student_id.clone(),
                    attendance.group_id.clone(),
                    attendance.date.to_rfc3339(),
                    attendance.status.as_str(),
                    attendance.notes.clone(),
                    attendance.id.clone(),
                ],
            )
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Attendance", &attendance.id));
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM attendance WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![id])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Attendance", id));
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Attendance>, DomainError> {
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance ORDER BY date DESC";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_attendance(&row)?);
        }
        Ok(results)
    }

    async fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Attendance>, DomainError> {
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE student_id = ?1 ORDER BY date DESC";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![student_id]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_attendance(&row)?);
        }
        Ok(results)
    }

    async fn find_by_group_and_date(
        &self,
        group_id: &str,
        date: DateTime<Utc>,
    ) -> Result<Vec<Attendance>, DomainError> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let sql = "SELECT id, student_id, group_id, date, status, notes, created_at
                   FROM attendance WHERE group_id = ?1 AND date LIKE ?2 ORDER BY date DESC";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![group_id, format!("{}%", date_str)]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_attendance(&row)?);
        }
        Ok(results)
    }

    async fn count_absences_by_student_and_group(
        &self,
        student_id: &str,
        group_id: &str,
    ) -> Result<i32, DomainError> {
        let sql = "SELECT COUNT(*) FROM attendance
                   WHERE student_id = ?1 AND group_id = ?2 AND status = 'absent'";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![student_id, group_id]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => {
                let count: i32 = row.get(0).map_err(|e| DomainError::Database(e.to_string()))?;
                Ok(count)
            }
            None => Ok(0),
        }
    }

    async fn count_absences_by_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<(String, i32)>, DomainError> {
        let sql = "SELECT student_id, COUNT(*) as absence_count FROM attendance
                   WHERE group_id = ?1 AND status = 'absent'
                   GROUP BY student_id";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![group_id]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            let student_id: String = row.get(0).map_err(|e| DomainError::Database(e.to_string()))?;
            let count: i32 = row.get(1).map_err(|e| DomainError::Database(e.to_string()))?;
            results.push((student_id, count));
        }
        Ok(results)
    }
}
