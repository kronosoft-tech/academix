//! Settings Use Cases

use crate::application::errors::ApplicationError;
use crate::application::ports::SettingsRepository;

/// Settings service
pub struct SettingsService<R: SettingsRepository> {
    settings_repository: R,
}

impl<R: SettingsRepository> SettingsService<R> {
    pub fn new(settings_repository: R) -> Self {
        Self {
            settings_repository,
        }
    }

    /// Get the absence threshold (default: 3)
    pub fn get_absence_threshold(&self) -> Result<i32, ApplicationError> {
        match self.settings_repository.get_setting("attendance_threshold")? {
            Some(value) => value
                .parse::<i32>()
                .map_err(|_| ApplicationError::Validation("Invalid threshold value".to_string())),
            None => Ok(3), // Default threshold
        }
    }

    /// Set the absence threshold (validates 1..=30)
    pub fn set_absence_threshold(&self, value: i32) -> Result<i32, ApplicationError> {
        if value < 1 || value > 30 {
            return Err(ApplicationError::Validation(
                "Threshold must be between 1 and 30".to_string(),
            ));
        }

        self.settings_repository
            .set_setting("attendance_threshold", &value.to_string())?;

        Ok(value)
    }
}