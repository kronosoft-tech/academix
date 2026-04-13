//! Employee Repository Ports (Trait Definitions)
//!
//! In-memory repository implementations for employees.

use crate::application::ports::employee::EmployeeRepository;
use crate::domain::entities::employee::{Employee, EmployeeStatus};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory implementation of EmployeeRepository
pub struct InMemoryEmployeeRepository {
    employees: Arc<RwLock<HashMap<String, Employee>>>,
    next_id: Arc<RwLock<u32>>,
}

impl InMemoryEmployeeRepository {
    pub fn new() -> Self {
        Self {
            employees: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    fn generate_id(&self) -> String {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("emp-{:03}", *counter);
        *counter += 1;
        id
    }

    pub fn seed_demo_employees(&self) {
        let demo_employees = vec![
            Employee::new(
                "emp-001".to_string(),
                crate::domain::entities::employee::DocumentType::CC,
                "12345678".to_string(),
                "María".to_string(),
                "García".to_string(),
                "maria@academix.edu".to_string(),
                "Directora Académica".to_string(),
                "Administración".to_string(),
                crate::domain::entities::employee::ContractType::Indefinite,
                5000.0,
                chrono::Utc::now(),
            ),
            Employee::new(
                "emp-002".to_string(),
                crate::domain::entities::employee::DocumentType::CC,
                "87654321".to_string(),
                "Carlos".to_string(),
                "López".to_string(),
                "carlos@academix.edu".to_string(),
                "Profesor de Matemáticas".to_string(),
                "Académico".to_string(),
                crate::domain::entities::employee::ContractType::Indefinite,
                3500.0,
                chrono::Utc::now(),
            ),
            Employee::new(
                "emp-003".to_string(),
                crate::domain::entities::employee::DocumentType::CC,
                "11223344".to_string(),
                "Ana".to_string(),
                "Martínez".to_string(),
                "ana@academix.edu".to_string(),
                "Coordinadora de Cursos".to_string(),
                "Operaciones".to_string(),
                crate::domain::entities::employee::ContractType::Fixed,
                2800.0,
                chrono::Utc::now(),
            ),
        ];

        let mut employees = self.employees.write().unwrap();
        for mut emp in demo_employees {
            emp.id = self.generate_id();
            employees.insert(emp.id.clone(), emp);
        }
    }
}

impl EmployeeRepository for InMemoryEmployeeRepository {
    fn create(&self, mut employee: Employee) -> Result<Employee, String> {
        if employee.id.is_empty() {
            employee.id = self.generate_id();
        }
        let mut employees = self.employees.write().unwrap();
        employees.insert(employee.id.clone(), employee.clone());
        Ok(employee)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Employee>, String> {
        let employees = self.employees.read().unwrap();
        Ok(employees.get(id).cloned())
    }

    fn get_by_document(&self, document_number: &str) -> Result<Option<Employee>, String> {
        let employees = self.employees.read().unwrap();
        Ok(employees
            .values()
            .find(|e| e.document_number == document_number)
            .cloned())
    }

    fn list(
        &self,
        status: Option<EmployeeStatus>,
        department: Option<&str>,
    ) -> Result<Vec<Employee>, String> {
        let employees = self.employees.read().unwrap();
        let mut result: Vec<Employee> = employees
            .values()
            .filter(|e| {
                let status_match = status.map_or(true, |s| e.status == s);
                let dept_match = department.map_or(true, |d| e.department == d);
                status_match && dept_match
            })
            .cloned()
            .collect();
        result.sort_by(|a, b| a.last_name.cmp(&b.last_name));
        Ok(result)
    }

    fn list_by_ids(&self, ids: &[String]) -> Result<Vec<Employee>, String> {
        let employees = self.employees.read().unwrap();
        let mut result: Vec<Employee> = employees
            .values()
            .filter(|e| ids.contains(&e.id))
            .cloned()
            .collect();
        result.sort_by(|a, b| a.last_name.cmp(&b.last_name));
        Ok(result)
    }

    fn update(&self, employee: Employee) -> Result<Employee, String> {
        let mut employees = self.employees.write().unwrap();
        if employees.contains_key(&employee.id) {
            employees.insert(employee.id.clone(), employee.clone());
            Ok(employee)
        } else {
            Err(format!("Employee not found: {}", employee.id))
        }
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut employees = self.employees.write().unwrap();
        if let Some(employee) = employees.get_mut(id) {
            employee.deactivate();
            Ok(true)
        } else {
            Err(format!("Employee not found: {}", id))
        }
    }

    fn count_by_status(&self, status: EmployeeStatus) -> Result<i64, String> {
        let employees = self.employees.read().unwrap();
        let count = employees.values().filter(|e| e.status == status).count() as i64;
        Ok(count)
    }

    fn get_total_salary_expense(&self, department: Option<&str>) -> Result<f64, String> {
        let employees = self.employees.read().unwrap();
        let total: f64 = employees
            .values()
            .filter(|e| e.is_active())
            .filter(|e| department.map_or(true, |d| e.department == d))
            .map(|e| e.base_salary)
            .sum();
        Ok(total)
    }
}
