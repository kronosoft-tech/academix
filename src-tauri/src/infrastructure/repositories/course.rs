//! In-Memory Course Repository

use std::collections::HashMap;
use std::sync::RwLock;

use crate::application::ports::CourseRepository;
use crate::domain::entities::Course;
use crate::domain::errors::DomainError;

/// In-memory course repository implementation
pub struct InMemoryCourseRepository {
    courses: RwLock<HashMap<String, Course>>,
    codes: RwLock<HashMap<String, String>>,
}

impl InMemoryCourseRepository {
    pub fn new() -> Self {
        Self {
            courses: RwLock::new(HashMap::new()),
            codes: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCourseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl CourseRepository for InMemoryCourseRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError> {
        let courses = self
            .courses
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(courses.get(id).cloned())
    }

    fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError> {
        let codes = self
            .codes
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let courses = self
            .courses
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(id) = codes.get(code) {
            Ok(courses.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn save(&self, course: &Course) -> Result<(), DomainError> {
        let mut courses = self
            .courses
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut codes = self
            .codes
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        courses.insert(course.id.clone(), course.clone());
        codes.insert(course.code.clone(), course.id.clone());

        Ok(())
    }

    fn update(&self, course: &Course) -> Result<(), DomainError> {
        let mut courses = self
            .courses
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if !courses.contains_key(&course.id) {
            return Err(DomainError::not_found("Course", &course.id));
        }

        courses.insert(course.id.clone(), course.clone());

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut courses = self
            .courses
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(course) = courses.get(id) {
            let mut codes = self
                .codes
                .write()
                .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
            codes.remove(&course.code);
            courses.remove(id);
        }

        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Course>, DomainError> {
        let courses = self
            .courses
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(courses.values().cloned().collect())
    }

    fn find_all_archived(&self) -> Result<Vec<Course>, DomainError> {
        let courses = self
            .courses
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(courses.values().cloned().collect())
    }

    fn restore(&self, _id: &str) -> Result<(), DomainError> {
        Ok(())
    }

    fn hard_delete(&self, id: &str) -> Result<(), DomainError> {
        let mut courses = self
            .courses
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(course) = courses.get(id) {
            let mut codes = self
                .codes
                .write()
                .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
            codes.remove(&course.code);
            courses.remove(id);
        }

        Ok(())
    }
}
