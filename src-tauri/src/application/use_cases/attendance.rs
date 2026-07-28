//! Attendance Use Cases

use crate::application::dto::{
    AttendanceDto, CreateAttendanceRequest, GroupAttendanceStats, StudentAbsenceCountDto,
    UpdateAttendanceRequest,
};
use crate::application::errors::ApplicationError;
use crate::application::ports::AttendanceRepository;
use crate::domain::entities::{Attendance, AttendanceStatus};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Attendance service
pub struct AttendanceService<R: AttendanceRepository> {
    attendance_repository: R,
}

impl<R: AttendanceRepository> AttendanceService<R> {
    pub fn new(attendance_repository: R) -> Self {
        Self {
            attendance_repository,
        }
    }

    /// Create a new attendance record
    pub async fn create(
        &self,
        request: CreateAttendanceRequest,
    ) -> Result<AttendanceDto, ApplicationError> {
        let date = DateTime::parse_from_rfc3339(&request.date)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(&request.date, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .map_err(|e| ApplicationError::Validation(format!("Invalid date format: {}", e)))?;

        let attendance = Attendance::new(
            Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            request.student_id,
            request.group_id,
            date,
            AttendanceStatus::from_str(&request.status).unwrap_or(AttendanceStatus::Present),
        );

        self.attendance_repository.save(&attendance).await?;

        Ok(AttendanceDto {
            id: attendance.id,
            student_id: attendance.student_id,
            group_id: attendance.group_id,
            date: attendance.date.to_rfc3339(),
            status: attendance.status.as_str().to_string(),
            notes: attendance.notes,
        })
    }

    /// Get attendance by ID
    pub async fn get_by_id(&self, id: &str) -> Result<AttendanceDto, ApplicationError> {
        let attendance = self
            .attendance_repository
            .find_by_id(id).await?
            .ok_or_else(|| ApplicationError::NotFound("Attendance not found".to_string()))?;

        Ok(AttendanceDto {
            id: attendance.id,
            student_id: attendance.student_id,
            group_id: attendance.group_id,
            date: attendance.date.to_rfc3339(),
            status: attendance.status.as_str().to_string(),
            notes: attendance.notes,
        })
    }

    /// List all attendance records
    pub async fn list(&self) -> Result<Vec<AttendanceDto>, ApplicationError> {
        let attendances = self.attendance_repository.find_all().await?;

        Ok(attendances
            .into_iter()
            .map(|a| AttendanceDto {
                id: a.id,
                student_id: a.student_id,
                group_id: a.group_id,
                date: a.date.to_rfc3339(),
                status: a.status.as_str().to_string(),
                notes: a.notes,
            })
            .collect())
    }

    /// List attendance by group and date
    pub async fn list_by_group_and_date(
        &self,
        group_id: &str,
        date: &str,
    ) -> Result<Vec<AttendanceDto>, ApplicationError> {
        let parsed_date = DateTime::parse_from_rfc3339(date)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            })
            .map_err(|e| ApplicationError::Validation(format!("Invalid date format: {}", e)))?;

        let attendances = self
            .attendance_repository
            .find_by_group_and_date(group_id, parsed_date).await?;

        Ok(attendances
            .into_iter()
            .map(|a| AttendanceDto {
                id: a.id,
                student_id: a.student_id,
                group_id: a.group_id,
                date: a.date.to_rfc3339(),
                status: a.status.as_str().to_string(),
                notes: a.notes,
            })
            .collect())
    }

    /// List attendance by student
    pub async fn list_by_student(
        &self,
        student_id: &str,
    ) -> Result<Vec<AttendanceDto>, ApplicationError> {
        let attendances = self.attendance_repository.find_by_student_id(student_id).await?;

        Ok(attendances
            .into_iter()
            .map(|a| AttendanceDto {
                id: a.id,
                student_id: a.student_id,
                group_id: a.group_id,
                date: a.date.to_rfc3339(),
                status: a.status.as_str().to_string(),
                notes: a.notes,
            })
            .collect())
    }

    /// Update attendance
    pub async fn update(
        &self,
        id: &str,
        request: UpdateAttendanceRequest,
    ) -> Result<AttendanceDto, ApplicationError> {
        let mut attendance = self
            .attendance_repository
            .find_by_id(id).await?
            .ok_or_else(|| ApplicationError::NotFound("Attendance not found".to_string()))?;

        if let Some(status) = request.status {
            attendance.status = AttendanceStatus::from_str(&status).unwrap_or(attendance.status);
        }

        if let Some(notes) = request.notes {
            attendance.notes = Some(notes);
        }

        self.attendance_repository.update(&attendance).await?;

        Ok(AttendanceDto {
            id: attendance.id,
            student_id: attendance.student_id,
            group_id: attendance.group_id,
            date: attendance.date.to_rfc3339(),
            status: attendance.status.as_str().to_string(),
            notes: attendance.notes,
        })
    }

    /// Delete attendance
    pub async fn delete(&self, id: &str) -> Result<(), ApplicationError> {
        self.attendance_repository.delete(id).await?;
        Ok(())
    }

    /// Get group attendance statistics
    pub async fn get_group_stats(
        &self,
        group_id: &str,
        total_students: i32,
    ) -> Result<GroupAttendanceStats, ApplicationError> {
        let attendances = self.attendance_repository.find_all().await?;

        // Filter by group
        let group_attendances: Vec<_> = attendances
            .into_iter()
            .filter(|a| a.group_id == group_id)
            .collect();

        let total_records = group_attendances.len() as i32;

        let present_count = group_attendances
            .iter()
            .filter(|a| a.status == AttendanceStatus::Present)
            .count() as i32;

        let absent_count = group_attendances
            .iter()
            .filter(|a| a.status == AttendanceStatus::Absent)
            .count() as i32;

        let late_count = group_attendances
            .iter()
            .filter(|a| a.status == AttendanceStatus::Late)
            .count() as i32;

        let excused_count = group_attendances
            .iter()
            .filter(|a| a.status == AttendanceStatus::Excused)
            .count() as i32;

        // Count unique sessions (dates)
        let total_sessions = group_attendances
            .iter()
            .map(|a| a.date.date_naive())
            .collect::<std::collections::HashSet<_>>()
            .len() as i32;

        // Calculate percentages
        let calc_percentage = |count: i32| -> f64 {
            if total_records == 0 {
                0.0
            } else {
                (count as f64 / total_records as f64) * 100.0
            }
        };

        Ok(GroupAttendanceStats {
            group_id: group_id.to_string(),
            total_records,
            present_count,
            present_percentage: calc_percentage(present_count),
            absent_count,
            absent_percentage: calc_percentage(absent_count),
            late_count,
            late_percentage: calc_percentage(late_count),
            excused_count,
            excused_percentage: calc_percentage(excused_count),
            total_students,
            total_sessions,
        })
    }

    /// Count absences for a student in a specific group
    pub async fn count_student_absences(
        &self,
        student_id: &str,
        group_id: &str,
    ) -> Result<i32, ApplicationError> {
        let count = self
            .attendance_repository
            .count_absences_by_student_and_group(student_id, group_id).await?;
        Ok(count)
    }

    /// Count absences for all students in a group
    pub async fn count_group_absences(
        &self,
        group_id: &str,
    ) -> Result<Vec<StudentAbsenceCountDto>, ApplicationError> {
        let counts = self
            .attendance_repository
            .count_absences_by_group(group_id).await?;

        Ok(counts
            .into_iter()
            .map(|(student_id, absence_count)| StudentAbsenceCountDto {
                student_id,
                absence_count,
            })
            .collect())
    }
}
