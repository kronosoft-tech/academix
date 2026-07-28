//! Course SQLite Repository
//!
//! Implements CourseRepository using SQLite.

use async_trait::async_trait;
use crate::application::ports::CourseRepository;
use crate::domain::entities::course::{Course, CourseStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::local_db;
use chrono::{DateTime, Utc};

/// SQLite implementation of CourseRepository
#[derive(Clone)]
pub struct SqliteCourseRepository;

impl SqliteCourseRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_course(row: &libsql::Row) -> Result<Course, DomainError> {
        let status_str: String = row.get(5).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(6).map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row.get(7).map_err(|e| DomainError::Database(e.to_string()))?;
        let price: f64 = row.get(8).unwrap_or(200000.0);
        let duration: i32 = row.get::<Option<i32>>(9).map_err(|e| DomainError::Database(e.to_string()))?.unwrap_or(0);

        Ok(Course {
            id: row.get(0).map_err(|e| DomainError::Database(e.to_string()))?,
            name: row.get(1).map_err(|e| DomainError::Database(e.to_string()))?,
            description: row.get(2).map_err(|e| DomainError::Database(e.to_string()))?,
            code: row.get(3).map_err(|e| DomainError::Database(e.to_string()))?,
            credits: row.get(4).map_err(|e| DomainError::Database(e.to_string()))?,
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

impl Default for SqliteCourseRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CourseRepository for SqliteCourseRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![id.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_course(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE code = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![code.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_course(&row)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, course: &Course) -> Result<(), DomainError> {
        eprintln!("[COURSE REPO] Saving course: id={}, name={}, status={}", course.id, course.name, course.status.as_str());
        let sql = "INSERT INTO courses (id, name, description, code, credits, status, created_at, updated_at, price, duration)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(
            sql,
            libsql::params![
                course.id.clone(),
                course.name.clone(),
                course.description.clone(),
                course.code.clone(),
                course.credits,
                course.status.as_str(),
                course.created_at.to_rfc3339(),
                course.updated_at.to_rfc3339(),
                course.price,
                course.duration,
            ],
        )
        .await
        .map_err(|e| {
            eprintln!("[COURSE REPO ERROR] Failed to insert: {}", e);
            DomainError::Validation(e.to_string())
        })?;

        eprintln!("[COURSE REPO] Course saved successfully");
        Ok(())
    }

    async fn update(&self, course: &Course) -> Result<(), DomainError> {
        let sql = "UPDATE courses 
                   SET name = ?1, description = ?2, code = ?3, credits = ?4, status = ?5, updated_at = ?6, price = ?7, duration = ?8
                   WHERE id = ?9";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    course.name.clone(),
                    course.description.clone(),
                    course.code.clone(),
                    course.credits.clone(),
                    course.status.as_str().to_string(),
                    Utc::now().to_rfc3339(),
                    course.price.clone(),
                    course.duration.clone(),
                    course.id.clone(),
                ],
            )
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Course", &course.id));
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE courses SET status = 'archived', updated_at = ?1 WHERE id = ?2";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![Utc::now().to_rfc3339(), id.to_string()])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Course", id));
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Course>, DomainError> {
        eprintln!("[COURSE REPO] Finding all courses (status != 'archived')");
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE status != 'archived' ORDER BY name";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_course(&row)?);
        }
        eprintln!("[COURSE REPO] Found {} courses", results.len());
        Ok(results)
    }

    async fn find_all_archived(&self) -> Result<Vec<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE status = 'archived' ORDER BY name";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_course(&row)?);
        }
        Ok(results)
    }

    async fn restore(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE courses SET status = 'draft', updated_at = ?1 WHERE id = ?2";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(sql, libsql::params![Utc::now().to_rfc3339(), id.to_string()])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }

    async fn hard_delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM courses WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(sql, libsql::params![id.to_string()])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }
}
