//! Course Use Cases

use crate::application::dto::{CourseDto, CreateCourseRequest, UpdateCourseRequest};
use crate::application::errors::ApplicationError;
use crate::application::ports::CourseRepository;
use crate::domain::entities::Course;
use uuid::Uuid;

/// Course service
pub struct CourseService<R: CourseRepository> {
    course_repository: R,
}

impl<R: CourseRepository> CourseService<R> {
    pub fn new(course_repository: R) -> Self {
        Self { course_repository }
    }

    /// Create a new course
    pub fn create(&self, request: CreateCourseRequest) -> Result<CourseDto, ApplicationError> {
        // Check if course code already exists
        if self
            .course_repository
            .find_by_code(&request.code)?
            .is_some()
        {
            return Err(ApplicationError::Conflict(
                "Course code already exists".to_string(),
            ));
        }

        let mut course = Course::new(
            Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            request.name,
            request.code,
            request.credits,
        );

        // Set description if provided
        if let Some(desc) = request.description {
            course.description = Some(desc);
        }

        // Set price and duration
        course.price = request.price;
        course.duration = request.duration;

        self.course_repository.save(&course)?;

        Ok(CourseDto {
            id: course.id,
            name: course.name,
            code: course.code,
            credits: course.credits,
            price: course.price,
            duration: course.duration,
            description: course.description,
            status: course.status.as_str().to_string(),
        })
    }

    /// Get course by ID
    pub fn get_by_id(&self, id: &str) -> Result<CourseDto, ApplicationError> {
        let course = self
            .course_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Course not found".to_string()))?;

        Ok(CourseDto {
            id: course.id,
            name: course.name,
            code: course.code,
            credits: course.credits,
            price: course.price,
            duration: course.duration,
            description: course.description,
            status: course.status.as_str().to_string(),
        })
    }

    /// List all courses
    pub fn list(&self) -> Result<Vec<CourseDto>, ApplicationError> {
        let courses = self.course_repository.find_all()?;

        Ok(courses
            .into_iter()
            .map(|c| CourseDto {
                id: c.id,
                name: c.name,
                code: c.code,
                credits: c.credits,
                price: c.price,
                duration: c.duration,
                description: c.description,
                status: c.status.as_str().to_string(),
            })
            .collect())
    }

    /// Update course
    pub fn update(
        &self,
        id: &str,
        request: UpdateCourseRequest,
    ) -> Result<CourseDto, ApplicationError> {
        let mut course = self
            .course_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Course not found".to_string()))?;

        if let Some(name) = request.name {
            course.name = name;
        }

        if let Some(description) = request.description {
            course.description = Some(description);
        }

        if let Some(credits) = request.credits {
            course.credits = credits;
        }

        if let Some(price) = request.price {
            course.price = price;
        }

        if let Some(duration) = request.duration {
            course.duration = duration;
        }

        course.updated_at = chrono::Utc::now();

        self.course_repository.update(&course)?;

        Ok(CourseDto {
            id: course.id,
            name: course.name,
            code: course.code,
            credits: course.credits,
            price: course.price,
            duration: course.duration,
            description: course.description,
            status: course.status.as_str().to_string(),
        })
    }

    /// Delete course (soft delete - marks as archived)
    pub fn delete(&self, id: &str) -> Result<(), ApplicationError> {
        self.course_repository.delete(id)?;
        Ok(())
    }

    /// List all archived courses
    pub fn list_archived(&self) -> Result<Vec<CourseDto>, ApplicationError> {
        let courses = self.course_repository.find_all_archived()?;
        Ok(courses
            .into_iter()
            .map(|c| CourseDto {
                id: c.id,
                name: c.name,
                code: c.code,
                credits: c.credits,
                price: c.price,
                duration: c.duration,
                description: c.description,
                status: c.status.as_str().to_string(),
            })
            .collect())
    }

    /// Restore an archived course
    pub fn restore(&self, id: &str) -> Result<(), ApplicationError> {
        self.course_repository.restore(id)?;
        Ok(())
    }

    /// Hard delete - permanently removes from database
    pub fn hard_delete(&self, id: &str) -> Result<(), ApplicationError> {
        self.course_repository.hard_delete(id)?;
        Ok(())
    }
}
