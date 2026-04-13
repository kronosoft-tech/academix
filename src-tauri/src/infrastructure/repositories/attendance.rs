//! In-Memory Attendance Repository

use std::collections::HashMap;
use std::sync::RwLock;

use crate::application::ports::AttendanceRepository;
use crate::domain::entities::Attendance;
use crate::domain::errors::DomainError;
use chrono::{DateTime, Utc};

/// In-memory attendance repository implementation
pub struct InMemoryAttendanceRepository {
    attendances: RwLock<HashMap<String, Attendance>>,
}

impl InMemoryAttendanceRepository {
    pub fn new() -> Self {
        Self {
            attendances: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryAttendanceRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AttendanceRepository for InMemoryAttendanceRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Attendance>, DomainError> {
        let attendances = self
            .attendances
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(attendances.get(id).cloned())
    }

    fn save(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let mut attendances = self
            .attendances
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        attendances.insert(attendance.id.clone(), attendance.clone());

        Ok(())
    }

    fn update(&self, attendance: &Attendance) -> Result<(), DomainError> {
        let mut attendances = self
            .attendances
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if !attendances.contains_key(&attendance.id) {
            return Err(DomainError::not_found("Attendance", &attendance.id));
        }

        attendances.insert(attendance.id.clone(), attendance.clone());

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut attendances = self
            .attendances
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        attendances.remove(id);

        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Attendance>, DomainError> {
        let attendances = self
            .attendances
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(attendances.values().cloned().collect())
    }

    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Attendance>, DomainError> {
        let attendances = self
            .attendances
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        Ok(attendances
            .values()
            .filter(|a| a.student_id == student_id)
            .cloned()
            .collect())
    }

    fn find_by_group_and_date(
        &self,
        group_id: &str,
        date: DateTime<Utc>,
    ) -> Result<Vec<Attendance>, DomainError> {
        let attendances = self
            .attendances
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        // Compare dates by year-month-day only
        let date_date = date.date_naive();

        Ok(attendances
            .values()
            .filter(|a| a.group_id == group_id && a.date.date_naive() == date_date)
            .cloned()
            .collect())
    }
}
