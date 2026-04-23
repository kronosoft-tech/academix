//! Employee SQLite Repository
//!
//! Implements EmployeeRepository using SQLite.

use crate::application::ports::EmployeeRepository;
use crate::domain::entities::employee::{
    AccountType, ContractType, DocumentType, Employee, EmployeeStatus, AFP,
};
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of EmployeeRepository
#[derive(Clone)]
pub struct SqliteEmployeeRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteEmployeeRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_employee(row: &rusqlite::Row<'_>) -> rusqlite::Result<Employee> {
        // Column indices match the SELECT query order:
        // 0: id, 1: user_id, 2: document_type, 3: document_number, 4: first_name,
        // 5: last_name, 6: email, 7: phone, 8: address, 9: position, 10: department,
        // 11: contract_type, 12: base_salary, 13: bank_name, 14: bank_account,
        // 15: account_type, 16: cci, 17: afp, 18: hire_date, 19: termination_date,
        // 20: status, 21: created_at, 22: updated_at
        let document_type_str: String = row.get(2)?;
        let contract_type_str: String = row.get(11)?;
        let account_type_str: Option<String> = row.get(15)?;
        let afp_str: Option<String> = row.get(17)?;
        let status_str: String = row.get(20)?;
        let hire_date_str: String = row.get(18)?;
        let termination_str: Option<String> = row.get(19)?;
        let created_str: String = row.get(21)?;
        let updated_str: String = row.get(22)?;

        let document_type = DocumentType::from_str(&document_type_str).unwrap_or(DocumentType::CC);
        let contract_type =
            ContractType::from_str(&contract_type_str).unwrap_or(ContractType::Indefinite);
        let account_type = account_type_str.and_then(|s| AccountType::from_str(&s));
        let afp = afp_str.and_then(|s| AFP::from_str(&s));
        let status = EmployeeStatus::from_str(&status_str).unwrap_or(EmployeeStatus::Active);

        Ok(Employee {
            id: row.get(0)?,
            user_id: row.get(1)?,
            document_type,
            document_number: row.get(3)?,
            first_name: row.get(4)?,
            last_name: row.get(5)?,
            email: row.get(6)?,
            phone: row.get(7)?,
            address: row.get(8)?,
            position: row.get(9)?,
            department: row.get(10)?,
            contract_type,
            base_salary: row.get(12)?,
            bank_name: row.get(13)?,
            bank_account: row.get(14)?,
            account_type,
            cci: row.get(15)?,
            afp,
            hire_date: DateTime::parse_from_rfc3339(&hire_date_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            termination_date: termination_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            status,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn document_type_to_string(dt: DocumentType) -> &'static str {
        dt.as_str()
    }

    fn contract_type_to_string(ct: ContractType) -> &'static str {
        ct.as_str()
    }

    fn account_type_to_string(at: Option<AccountType>) -> Option<String> {
        at.map(|a| a.as_str().to_string())
    }

    fn afp_to_string(afp: Option<AFP>) -> Option<String> {
        afp.map(|a| a.as_str().to_string())
    }

    fn status_to_string(status: EmployeeStatus) -> &'static str {
        status.as_str()
    }
}

impl EmployeeRepository for SqliteEmployeeRepository {
    fn create(&self, employee: Employee) -> Result<Employee, String> {
        let sql = "INSERT INTO employees (
                      id, user_id, document_type, document_number, first_name, last_name,
                      email, phone, address, position, department, contract_type,
                      base_salary, bank_name, bank_account, account_type, cci, afp,
                      hire_date, termination_date, status, created_at, updated_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let termination_date = employee.termination_date.map(|dt| dt.to_rfc3339());

        self.pool
            .execute(
                sql,
                &[
                    &employee.id,
                    &employee.user_id,
                    &Self::document_type_to_string(employee.document_type),
                    &employee.document_number,
                    &employee.first_name,
                    &employee.last_name,
                    &employee.email,
                    &employee.phone,
                    &employee.address,
                    &employee.position,
                    &employee.department,
                    &Self::contract_type_to_string(employee.contract_type),
                    &employee.base_salary.to_string(),
                    &employee.bank_name,
                    &employee.bank_account,
                    &Self::account_type_to_string(employee.account_type),
                    &employee.cci,
                    &Self::afp_to_string(employee.afp),
                    &employee.hire_date.to_rfc3339(),
                    &termination_date,
                    &Self::status_to_string(employee.status),
                    &employee.created_at.to_rfc3339(),
                    &employee.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(employee)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Employee>, String> {
        let sql = "SELECT id, user_id, document_type, document_number, first_name, last_name,
                         email, phone, address, position, department, contract_type,
                         base_salary, bank_name, bank_account, account_type, cci, afp,
                         hire_date, termination_date, status, created_at, updated_at
                  FROM employees WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_employee)
            .map_err(|e| e.to_string())
    }

    fn get_by_document(&self, document_number: &str) -> Result<Option<Employee>, String> {
        let sql = "SELECT id, user_id, document_type, document_number, first_name, last_name,
                         email, phone, address, position, department, contract_type,
                         base_salary, bank_name, bank_account, account_type, cci, afp,
                         hire_date, termination_date, status, created_at, updated_at
                  FROM employees WHERE document_number = ?";

        self.pool
            .query_row(sql, &[&document_number], Self::row_to_employee)
            .map_err(|e| e.to_string())
    }

    fn list(
        &self,
        status: Option<EmployeeStatus>,
        department: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<Employee>, String> {
        let mut sql = "SELECT id, user_id, document_type, document_number, first_name, last_name,
                          email, phone, address, position, department, contract_type,
                          base_salary, bank_name, bank_account, account_type, cci, afp,
                          hire_date, termination_date, status, created_at, updated_at
                    FROM employees WHERE 1=1"
            .to_string();
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            params.push(Self::status_to_string(s).to_string());
        }

        if let Some(d) = department {
            sql.push_str(" AND department = ?");
            params.push(d.to_string());
        }

        if let Some(q) = search {
            sql.push_str(" AND (first_name LIKE ? OR last_name LIKE ? OR document_number LIKE ? OR email LIKE ?)");
            let search_pattern = format!("%{}%", q);
            params.push(search_pattern.clone());
            params.push(search_pattern.clone());
            params.push(search_pattern.clone());
            params.push(search_pattern);
        }

        sql.push_str(" ORDER BY last_name, first_name");

        self.pool
            .query_with_vec(&sql, params, Self::row_to_employee)
            .map_err(|e| e.to_string())
    }

    fn list_by_ids(&self, ids: &[String]) -> Result<Vec<Employee>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT id, user_id, document_type, document_number, first_name, last_name,
                    email, phone, address, position, department, contract_type,
                    base_salary, bank_name, bank_account, account_type, cci, afp,
                    hire_date, termination_date, status, created_at, updated_at
              FROM employees WHERE id IN ({}) ORDER BY last_name, first_name",
            placeholders.join(", ")
        );

        let _params: Vec<String> = ids.iter().map(|s| s.clone()).collect();
        self.pool
            .query(&sql, &[], Self::row_to_employee)
            .map_err(|e| e.to_string())
    }

    fn update(&self, employee: Employee) -> Result<Employee, String> {
        let sql = "UPDATE employees 
                  SET user_id = ?, document_type = ?, document_number = ?, 
                      first_name = ?, last_name = ?, email = ?, phone = ?, address = ?,
                      position = ?, department = ?, contract_type = ?, base_salary = ?,
                      bank_name = ?, bank_account = ?, account_type = ?, cci = ?, afp = ?,
                      hire_date = ?, termination_date = ?, status = ?, updated_at = ?
                  WHERE id = ?";

        let termination_date = employee.termination_date.map(|dt| dt.to_rfc3339());

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &employee.user_id,
                    &Self::document_type_to_string(employee.document_type),
                    &employee.document_number,
                    &employee.first_name,
                    &employee.last_name,
                    &employee.email,
                    &employee.phone,
                    &employee.address,
                    &employee.position,
                    &employee.department,
                    &Self::contract_type_to_string(employee.contract_type),
                    &employee.base_salary.to_string(),
                    &employee.bank_name,
                    &employee.bank_account,
                    &Self::account_type_to_string(employee.account_type),
                    &employee.cci,
                    &Self::afp_to_string(employee.afp),
                    &employee.hire_date.to_rfc3339(),
                    &termination_date,
                    &Self::status_to_string(employee.status),
                    &Utc::now().to_rfc3339(),
                    &employee.id,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("Employee not found: {}", employee.id));
        }

        Ok(employee)
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "UPDATE employees SET status = 'inactive', updated_at = ? WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&Utc::now().to_rfc3339(), &id])
            .map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn count_by_status(&self, status: EmployeeStatus) -> Result<i64, String> {
        let sql = "SELECT COUNT(*) FROM employees WHERE status = ?";
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let count: i64 = conn
            .query_row(sql, [Self::status_to_string(status)], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(count)
    }

    fn get_total_salary_expense(&self, department: Option<&str>) -> Result<f64, String> {
        let mut sql = "SELECT COALESCE(SUM(base_salary), 0) FROM employees WHERE status = 'active'"
            .to_string();
        let mut params: Vec<String> = vec![];

        if let Some(d) = department {
            sql.push_str(" AND department = ?");
            params.push(d.to_string());
        }

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = if params.is_empty() {
            conn.query_row(&sql, [], |row| row.get(0))
        } else {
            conn.query_row(&sql, &[&params[0]], |row| row.get(0))
        }
        .map_err(|e| e.to_string())?;

        Ok(total)
    }
}
