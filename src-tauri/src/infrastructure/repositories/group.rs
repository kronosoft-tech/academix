//! In-Memory Group Repository

use std::collections::HashMap;
use std::sync::RwLock;

use crate::application::ports::GroupRepository;
use crate::domain::entities::Group;
use crate::domain::errors::DomainError;

/// In-memory group repository implementation
pub struct InMemoryGroupRepository {
    groups: RwLock<HashMap<String, Group>>,
    course_groups: RwLock<HashMap<String, Vec<String>>>,
    professor_groups: RwLock<HashMap<String, Vec<String>>>,
}

impl InMemoryGroupRepository {
    pub fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
            course_groups: RwLock::new(HashMap::new()),
            professor_groups: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryGroupRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupRepository for InMemoryGroupRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Group>, DomainError> {
        let groups = self
            .groups
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(groups.get(id).cloned())
    }

    fn find_by_course_id(&self, course_id: &str) -> Result<Vec<Group>, DomainError> {
        let course_groups = self
            .course_groups
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let groups = self
            .groups
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        let mut result = Vec::new();
        if let Some(group_ids) = course_groups.get(course_id) {
            for id in group_ids {
                if let Some(group) = groups.get(id) {
                    result.push(group.clone());
                }
            }
        }

        Ok(result)
    }

    fn find_by_professor_id(&self, professor_id: &str) -> Result<Vec<Group>, DomainError> {
        let professor_groups = self
            .professor_groups
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let groups = self
            .groups
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        let mut result = Vec::new();
        if let Some(group_ids) = professor_groups.get(professor_id) {
            for id in group_ids {
                if let Some(group) = groups.get(id) {
                    result.push(group.clone());
                }
            }
        }

        Ok(result)
    }

    fn save(&self, group: &Group) -> Result<(), DomainError> {
        let mut groups = self
            .groups
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut course_groups = self
            .course_groups
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut professor_groups = self
            .professor_groups
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        groups.insert(group.id.clone(), group.clone());

        course_groups
            .entry(group.course_id.clone())
            .or_insert_with(Vec::new)
            .push(group.id.clone());

        // Only add to professor_groups if professor_id is set
        if let Some(ref prof_id) = group.professor_id {
            professor_groups
                .entry(prof_id.clone())
                .or_insert_with(Vec::new)
                .push(group.id.clone());
        }

        Ok(())
    }

    fn update(&self, group: &Group) -> Result<(), DomainError> {
        let mut groups = self
            .groups
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if !groups.contains_key(&group.id) {
            return Err(DomainError::not_found("Group", &group.id));
        }

        groups.insert(group.id.clone(), group.clone());

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut groups = self
            .groups
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        groups.remove(id);
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Group>, DomainError> {
        let groups = self
            .groups
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(groups.values().cloned().collect())
    }

    fn has_capacity(&self, group_id: &str) -> Result<bool, DomainError> {
        let groups = self
            .groups
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        match groups.get(group_id) {
            Some(group) => Ok(group.current_students < group.max_students),
            None => Ok(false),
        }
    }

    fn increment_students(&self, group_id: &str) -> Result<(), DomainError> {
        let mut groups = self
            .groups
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        match groups.get_mut(group_id) {
            Some(group) => {
                group.current_students += 1;
                Ok(())
            }
            None => Err(DomainError::not_found("Group", group_id)),
        }
    }

    fn decrement_students(&self, group_id: &str) -> Result<(), DomainError> {
        let mut groups = self
            .groups
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        match groups.get_mut(group_id) {
            Some(group) => {
                if group.current_students > 0 {
                    group.current_students -= 1;
                }
                Ok(())
            }
            None => Err(DomainError::not_found("Group", group_id)),
        }
    }
}
