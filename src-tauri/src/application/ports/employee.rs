//! Employee Repository Port
//!
//! Port interface for employee repository operations.

use crate::domain::entities::employee::{Employee, EmployeeStatus};

/// Employee repository trait (port)
pub trait EmployeeRepository: Send + Sync {
    /// Create a new employee
    fn create(&self, employee: Employee) -> Result<Employee, String>;

    /// Get employee by ID
    fn get_by_id(&self, id: &str) -> Result<Option<Employee>, String>;

    /// Get employee by document number
    fn get_by_document(&self, document_number: &str) -> Result<Option<Employee>, String>;

    /// List all employees with optional filters
    fn list(
        &self,
        status: Option<EmployeeStatus>,
        department: Option<&str>,
    ) -> Result<Vec<Employee>, String>;

    /// List employees by IDs
    fn list_by_ids(&self, ids: &[String]) -> Result<Vec<Employee>, String>;

    /// Update employee
    fn update(&self, employee: Employee) -> Result<Employee, String>;

    /// Delete (soft delete - deactivate) employee
    fn delete(&self, id: &str) -> Result<bool, String>;

    /// Count employees by status
    fn count_by_status(&self, status: EmployeeStatus) -> Result<i64, String>;

    /// Get total salary expense
    fn get_total_salary_expense(&self, department: Option<&str>) -> Result<f64, String>;
}
