//! Settings Repository Port

use crate::domain::errors::DomainError;

/// Settings repository port - defines operations for application settings persistence
pub trait SettingsRepository: Send + Sync {
    /// Get a setting value by key
    fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError>;

    /// Set a setting value (insert or update)
    fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError>;
}