//! Liability Repository Port

use crate::infrastructure::repositories::liability::Liability;

/// Trait for liability repository operations
pub trait LiabilityRepository: Send + Sync {
    /// Create a new liability
    fn create(&self, liability: &Liability) -> Result<(), String>;

    /// List all liabilities (not paid)
    fn list(&self) -> Result<Vec<Liability>, String>;

    /// Get total liability amount by type
    fn get_total_by_type(&self, liability_type: &str) -> Result<f64, String>;

    /// Get total liability amount
    fn get_total(&self) -> Result<f64, String>;
}