//! Employee Service
//!
//! Use case for employee management operations.

use crate::application::dto::employee::{
    CreateEmployeeRequest, DepartmentSummary, EmployeeDto, EmployeeListItem, EmployeeSummary,
    UpdateEmployeeRequest,
};
use crate::application::ports::employee::EmployeeRepository;
use crate::domain::entities::employee::Employee;
use chrono::Utc;

/// Employee service - orchestrates employee operations
pub struct EmployeeService<R: EmployeeRepository> {
    repository: R,
}

impl<R: EmployeeRepository> EmployeeService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Create a new employee
    pub fn create_employee(&self, request: CreateEmployeeRequest) -> Result<EmployeeDto, String> {
        // Validate required fields
        if request.first_name.trim().is_empty() {
            return Err("First name is required".to_string());
        }
        if request.last_name.trim().is_empty() {
            return Err("Last name is required".to_string());
        }
        if request.email.trim().is_empty() {
            return Err("Email is required".to_string());
        }
        if request.position.trim().is_empty() {
            return Err("Position is required".to_string());
        }
        if request.department.trim().is_empty() {
            return Err("Department is required".to_string());
        }
        if request.base_salary <= 0.0 {
            return Err("Base salary must be greater than 0".to_string());
        }

        // Check for duplicate document number
        if let Some(_) = self.repository.get_by_document(&request.document_number)? {
            return Err(format!(
                "Employee with document {} already exists",
                request.document_number
            ));
        }

        let employee = Employee::new(
            String::new(), // Will generate ID in repository
            request.document_type,
            request.document_number,
            request.first_name,
            request.last_name,
            request.email,
            request.position,
            request.department,
            request.contract_type,
            request.base_salary,
            Utc::now(),
        );

        let created = self.repository.create(employee)?;
        Ok(EmployeeDto::from(created))
    }

    /// Get employee by ID
    pub fn get_employee(&self, id: &str) -> Result<Option<EmployeeDto>, String> {
        let employee = self.repository.get_by_id(id)?;
        Ok(employee.map(EmployeeDto::from))
    }

    /// List employees with optional filters
    pub fn list_employees(
        &self,
        status: Option<crate::domain::entities::employee::EmployeeStatus>,
        department: Option<&str>,
    ) -> Result<Vec<EmployeeListItem>, String> {
        let employees = self.repository.list(status, department)?;
        Ok(employees.into_iter().map(EmployeeListItem::from).collect())
    }

    /// Update employee
    pub fn update_employee(
        &self,
        id: &str,
        request: UpdateEmployeeRequest,
    ) -> Result<EmployeeDto, String> {
        let mut employee = self
            .repository
            .get_by_id(id)?
            .ok_or_else(|| format!("Employee not found: {}", id))?;

        // Apply updates
        if let Some(first_name) = request.first_name {
            employee.first_name = first_name;
        }
        if let Some(last_name) = request.last_name {
            employee.last_name = last_name;
        }
        if let Some(email) = request.email {
            employee.email = email;
        }
        if let Some(phone) = request.phone {
            employee.phone = Some(phone);
        }
        if let Some(address) = request.address {
            employee.address = Some(address);
        }
        if let Some(position) = request.position {
            employee.position = position;
        }
        if let Some(department) = request.department {
            employee.department = department;
        }
        if let Some(contract_type) = request.contract_type {
            employee.contract_type = contract_type;
        }
        if let Some(base_salary) = request.base_salary {
            if base_salary <= 0.0 {
                return Err("Base salary must be greater than 0".to_string());
            }
            employee.base_salary = base_salary;
        }
        if let Some(bank_name) = request.bank_name {
            employee.bank_name = Some(bank_name);
        }
        if let Some(bank_account) = request.bank_account {
            employee.bank_account = Some(bank_account);
        }
        if let Some(account_type) = request.account_type {
            employee.account_type = Some(account_type);
        }
        if let Some(cci) = request.cci {
            employee.cci = Some(cci);
        }
        if let Some(afp) = request.afp {
            employee.afp = Some(afp);
        }
        if let Some(status) = request.status {
            match status {
                crate::domain::entities::employee::EmployeeStatus::Terminated => {
                    employee.terminate(Utc::now());
                }
                crate::domain::entities::employee::EmployeeStatus::Inactive => {
                    employee.deactivate();
                }
                _ => {}
            }
        }

        let updated = self.repository.update(employee)?;
        Ok(EmployeeDto::from(updated))
    }

    /// Delete (deactivate) employee
    pub fn delete_employee(&self, id: &str) -> Result<bool, String> {
        self.repository.delete(id)
    }

    /// Get employee summary for dashboard
    pub fn get_summary(&self) -> Result<EmployeeSummary, String> {
        let all_employees = self.repository.list(None, None)?;

        let active = all_employees.iter().filter(|e| e.is_active()).count() as i64;
        let total = all_employees.len() as i64;
        let inactive = total - active;

        let total_salary: f64 = all_employees
            .iter()
            .filter(|e| e.is_active())
            .map(|e| e.base_salary)
            .sum();

        // Group by department
        let mut dept_map: std::collections::HashMap<String, (i64, f64)> =
            std::collections::HashMap::new();
        for emp in all_employees.iter().filter(|e| e.is_active()) {
            let entry = dept_map.entry(emp.department.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += emp.base_salary;
        }

        let by_department: Vec<DepartmentSummary> = dept_map
            .into_iter()
            .map(|(dept, (count, salary))| DepartmentSummary {
                department: dept,
                count,
                total_salary: salary,
            })
            .collect();

        Ok(EmployeeSummary {
            total_employees: total,
            active_employees: active,
            inactive_employees: inactive,
            total_salary_expense: total_salary,
            by_department,
        })
    }
}
