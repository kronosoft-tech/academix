//! In-Memory Student Repository

use std::collections::HashMap;
use std::sync::RwLock;

use crate::application::ports::StudentRepository;
use crate::domain::entities::Student;
use crate::domain::errors::DomainError;

/// In-memory student repository implementation
pub struct InMemoryStudentRepository {
    students: RwLock<HashMap<String, Student>>,
    user_ids: RwLock<HashMap<String, String>>,
}

impl InMemoryStudentRepository {
    pub fn new() -> Self {
        Self {
            students: RwLock::new(HashMap::new()),
            user_ids: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryStudentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl StudentRepository for InMemoryStudentRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Student>, DomainError> {
        let students = self
            .students
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(students.get(id).cloned())
    }

    fn find_by_user_id(&self, user_id: &str) -> Result<Option<Student>, DomainError> {
        let user_ids = self
            .user_ids
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let students = self
            .students
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(id) = user_ids.get(user_id) {
            Ok(students.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn save(&self, student: &Student) -> Result<(), DomainError> {
        let mut students = self
            .students
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut user_ids = self
            .user_ids
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        students.insert(student.id.clone(), student.clone());
        user_ids.insert(student.user_id.clone(), student.id.clone());

        Ok(())
    }

    fn update(&self, student: &Student) -> Result<(), DomainError> {
        let mut students = self
            .students
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if !students.contains_key(&student.id) {
            return Err(DomainError::not_found("Student", &student.id));
        }

        students.insert(student.id.clone(), student.clone());

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut students = self
            .students
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if let Some(student) = students.get(id) {
            let mut user_ids = self
                .user_ids
                .write()
                .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
            user_ids.remove(&student.user_id);
            students.remove(id);
        }

        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Student>, DomainError> {
        let students = self
            .students
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(students.values().cloned().collect())
    }
}
