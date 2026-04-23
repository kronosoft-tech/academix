//! Equity Repository Port

use crate::infrastructure::repositories::liability::Equity;

/// Trait for equity repository operations
pub trait EquityRepository: Send + Sync {
    /// Create a new equity
    fn create(&self, equity: &Equity) -> Result<(), String>;

    /// List all equities
    fn list(&self) -> Result<Vec<Equity>, String>;

    /// Get total equity amount by type
    fn get_total_by_type(&self, equity_type: &str) -> Result<f64, String>;

    /// Get total equity amount
    fn get_total(&self) -> Result<f64, String>;
}