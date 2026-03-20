//! Group Use Cases

use crate::application::dto::{CreateGroupRequest, GroupDto, UpdateGroupRequest};
use crate::application::errors::ApplicationError;
use crate::application::ports::GroupRepository;
use crate::domain::entities::Group;
use uuid::Uuid;

/// Group service
pub struct GroupService<R: GroupRepository> {
    group_repository: R,
}

impl<R: GroupRepository> GroupService<R> {
    pub fn new(group_repository: R) -> Self {
        Self { group_repository }
    }

    fn group_to_dto(&self, group: &Group) -> GroupDto {
        GroupDto {
            id: group.id.clone(),
            course_id: group.course_id.clone(),
            name: group.name.clone(),
            professor_id: group.professor_id.clone().unwrap_or_default(),
            schedule: group.schedule.clone(),
            days: group.days.clone(),
            start_time: group.start_time.clone(),
            end_time: group.end_time.clone(),
            max_students: group.max_students,
            current_students: group.current_students,
            status: group.status.as_str().to_string(),
        }
    }

    /// Create a new group
    pub fn create(&self, request: CreateGroupRequest) -> Result<GroupDto, ApplicationError> {
        let group = Group::new(
            Uuid::new_v4().to_string(),
            request.course_id,
            request.name,
            request.professor_id,
            request.schedule,
            request.days,
            request.start_time,
            request.end_time,
            request.max_students,
        );

        self.group_repository.save(&group)?;

        Ok(self.group_to_dto(&group))
    }

    /// Get group by ID
    pub fn get_by_id(&self, id: &str) -> Result<GroupDto, ApplicationError> {
        let group = self
            .group_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Group not found".to_string()))?;

        Ok(self.group_to_dto(&group))
    }

    /// List all groups
    pub fn list(&self) -> Result<Vec<GroupDto>, ApplicationError> {
        let groups = self.group_repository.find_all()?;

        Ok(groups.iter().map(|g| self.group_to_dto(g)).collect())
    }

    /// Update group
    pub fn update(
        &self,
        id: &str,
        request: UpdateGroupRequest,
    ) -> Result<GroupDto, ApplicationError> {
        let mut group = self
            .group_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Group not found".to_string()))?;

        if let Some(name) = request.name {
            group.name = name;
        }

        if let Some(professor_id) = request.professor_id {
            group.professor_id = Some(professor_id);
        }

        if let Some(schedule) = request.schedule {
            group.schedule = Some(schedule);
        }

        if let Some(days) = request.days {
            group.days = Some(days);
        }

        if let Some(start_time) = request.start_time {
            group.start_time = Some(start_time);
        }

        if let Some(end_time) = request.end_time {
            group.end_time = Some(end_time);
        }

        if let Some(max_students) = request.max_students {
            group.max_students = max_students;
        }

        if let Some(status) = request.status {
            group.status = crate::domain::entities::group::GroupStatus::from_str(&status)
                .unwrap_or(group.status);
        }

        self.group_repository.update(&group)?;

        Ok(self.group_to_dto(&group))
    }

    /// Delete group (soft delete)
    pub fn delete(&self, id: &str) -> Result<(), ApplicationError> {
        self.group_repository.delete(id)?;
        Ok(())
    }
}
