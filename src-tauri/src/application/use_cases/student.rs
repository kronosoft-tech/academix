//! Student Use Cases

use crate::application::dto::{CreateStudentRequest, StudentDto, UpdateStudentRequest};
use crate::application::errors::ApplicationError;
use crate::application::ports::{GroupRepository, StudentRepository};
use crate::domain::entities::Student;
use chrono::{DateTime, Utc};
use uuid::Uuid;

fn parse_birth_date(date_str: &Option<String>) -> Option<DateTime<Utc>> {
    date_str.as_ref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            // Try RFC3339 first (full datetime), then try YYYY-MM-DD (date only from frontend)
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|| {
                    // Parse YYYY-MM-DD format from HTML date input
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                        .ok()
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                })
        }
    })
}

fn validate_guardian_fields(
    birth_date: &Option<DateTime<Utc>>,
    guardian_name: &Option<String>,
    _guardian_document: &Option<String>,
    guardian_phone: &Option<String>,
) -> Result<(), ApplicationError> {
    if let Some(birth) = birth_date {
        let now = Utc::now();
        let age = now.signed_duration_since(*birth).num_days() / 365;

        if age < 18 {
            if guardian_name.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                return Err(ApplicationError::Validation(
                    "Guardian name is required for students under 18".to_string(),
                ));
            }
            if guardian_phone
                .as_ref()
                .map(|s| s.is_empty())
                .unwrap_or(true)
            {
                return Err(ApplicationError::Validation(
                    "Guardian phone is required for students under 18".to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Student service
pub struct StudentService<R: StudentRepository, G: GroupRepository> {
    student_repository: R,
    group_repository: G,
}

impl<R: StudentRepository, G: GroupRepository> StudentService<R, G> {
    pub fn new(student_repository: R, group_repository: G) -> Self {
        Self {
            student_repository,
            group_repository,
        }
    }

    /// Create a new student
    pub fn create(&self, request: CreateStudentRequest) -> Result<StudentDto, ApplicationError> {
        let birth_date = parse_birth_date(&request.birth_date);

        validate_guardian_fields(
            &birth_date,
            &request.guardian_name,
            &request.guardian_document,
            &request.guardian_phone,
        )?;

        if let Some(ref group_id) = request.group_id {
            let has_capacity = self
                .group_repository
                .has_capacity(group_id)
                .map_err(|e| ApplicationError::Validation(e.to_string()))?;

            if !has_capacity {
                return Err(ApplicationError::Validation(
                    "El grupo seleccionado está lleno. Selecciona otro grupo.".to_string(),
                ));
            }
        }

        let mut student = Student::new(
            Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            request.user_id,
            request.first_name,
            request.last_name,
            request.document_type,
            request.document_number,
            request.email,
        );

        student.phone = request.phone;
        student.address = request.address;
        student.birth_date = birth_date;
        student.guardian_name = request.guardian_name.filter(|s| !s.is_empty());
        student.guardian_document = request.guardian_document.filter(|s| !s.is_empty());
        student.guardian_phone = request.guardian_phone.filter(|s| !s.is_empty());
        student.course_id = request.course_id;
        student.group_id = request.group_id.clone();

        self.student_repository.save(&student)?;

        if let Some(ref group_id) = student.group_id {
            self.group_repository
                .increment_students(group_id)
                .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        }

        Ok(self.student_to_dto(&student))
    }

    fn student_to_dto(&self, student: &Student) -> StudentDto {
        StudentDto {
            id: student.id.clone(),
            user_id: student.user_id.clone(),
            first_name: student.first_name.clone(),
            last_name: student.last_name.clone(),
            document_type: student.document_type.clone(),
            document_number: student.document_number.clone(),
            email: student.email.clone(),
            phone: student.phone.clone(),
            address: student.address.clone(),
            birth_date: student.birth_date.map(|dt| dt.to_rfc3339()),
            guardian_name: student.guardian_name.clone(),
            guardian_document: student.guardian_document.clone(),
            guardian_phone: student.guardian_phone.clone(),
            course_id: student.course_id.clone(),
            group_id: student.group_id.clone(),
            course_name: None,
            group_name: None,
            active: student.active,
            created_at: student.created_at.to_rfc3339(),
            updated_at: student.updated_at.to_rfc3339(),
        }
    }

    /// Get student by ID
    pub fn get_by_id(&self, id: &str) -> Result<StudentDto, ApplicationError> {
        let student = self
            .student_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Student not found".to_string()))?;

        Ok(self.student_to_dto(&student))
    }

    /// List all students
    pub fn list(&self) -> Result<Vec<StudentDto>, ApplicationError> {
        let students = self.student_repository.find_all()?;

        Ok(students
            .into_iter()
            .map(|s| self.student_to_dto(&s))
            .collect())
    }

    /// Update student
    pub fn update(
        &self,
        id: &str,
        request: UpdateStudentRequest,
    ) -> Result<StudentDto, ApplicationError> {
        let mut student = self
            .student_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Student not found".to_string()))?;

        let birth_date = parse_birth_date(&request.birth_date).or(student.birth_date);

        let guardian_name = request
            .guardian_name
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
            .or(student.guardian_name.clone());
        let guardian_document = request
            .guardian_document
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
            .or(student.guardian_document.clone());
        let guardian_phone = request
            .guardian_phone
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
            .or(student.guardian_phone.clone());

        let course_id = request.course_id.clone();
        let new_group_id = request.group_id.clone();

        let previous_group_id = student.group_id.clone();

        if new_group_id != previous_group_id {
            if let Some(ref new_gid) = new_group_id {
                let has_capacity = self
                    .group_repository
                    .has_capacity(new_gid)
                    .map_err(|e| ApplicationError::Validation(e.to_string()))?;

                if !has_capacity {
                    return Err(ApplicationError::Validation(
                        "El grupo seleccionado está lleno. Selecciona otro grupo.".to_string(),
                    ));
                }
            }
        }

        validate_guardian_fields(
            &birth_date,
            &guardian_name,
            &guardian_document,
            &guardian_phone,
        )?;

        student.update(
            request.first_name.clone(),
            request.last_name.clone(),
            request.email.clone(),
            request.phone.clone(),
            request.address.clone(),
            birth_date,
            guardian_name,
            guardian_document,
            guardian_phone,
            course_id,
            new_group_id.clone(),
        );

        self.student_repository.update(&student)?;

        if new_group_id != previous_group_id {
            if let Some(ref old_gid) = previous_group_id {
                self.group_repository
                    .decrement_students(old_gid)
                    .map_err(|e| ApplicationError::Validation(e.to_string()))?;
            }

            if let Some(ref new_gid) = new_group_id {
                self.group_repository
                    .increment_students(new_gid)
                    .map_err(|e| ApplicationError::Validation(e.to_string()))?;
            }
        }

        Ok(self.student_to_dto(&student))
    }

    /// Delete student (soft delete)
    pub fn delete(&self, id: &str) -> Result<(), ApplicationError> {
        let student = self
            .student_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Student not found".to_string()))?;

        if let Some(ref group_id) = student.group_id {
            self.group_repository
                .decrement_students(group_id)
                .map_err(|e| ApplicationError::Validation(e.to_string()))?;
        }

        self.student_repository.delete(id)?;
        Ok(())
    }
}
