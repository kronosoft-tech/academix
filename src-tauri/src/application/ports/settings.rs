//! Settings Repository Port

use async_trait::async_trait;
use crate::domain::errors::DomainError;

/// Settings repository port - defines operations for application settings persistence
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Get a setting value by key
    async fn get_setting(&self, key: &str) -> Result<Option<String>, DomainError>;

    /// Set a setting value (insert or update)
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), DomainError>;
}
