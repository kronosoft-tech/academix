//! Employee Entity - Domain Model
//!
//! Pure domain entity for employee management with payroll data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Document types for employee identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentType {
    CC,  // Cédula de Ciudadanía
    CE,  // Carné de Extranjería
    RUC, // Registro Único de Contribuyentes
    NIT, // Número de Identificación Tributaria
    Passport,
}

impl DocumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentType::CC => "CC",
            DocumentType::CE => "CE",
            DocumentType::RUC => "RUC",
            DocumentType::NIT => "NIT",
            DocumentType::Passport => "PASSPORT",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "CC" => Some(DocumentType::CC),
            "CE" => Some(DocumentType::CE),
            "RUC" => Some(DocumentType::RUC),
            "NIT" => Some(DocumentType::NIT),
            "PASSPORT" => Some(DocumentType::Passport),
            _ => None,
        }
    }
}

/// Contract types for employment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractType {
    Fixed,      // Contrato fijo/determinado
    Indefinite, // Contrato indefinido
    Hourly,     // Contrato por horas
    Services,   // Contrato por prestación de servicios
}

impl ContractType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContractType::Fixed => "fixed",
            ContractType::Indefinite => "indefinite",
            ContractType::Hourly => "hourly",
            ContractType::Services => "services",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fixed" => Some(ContractType::Fixed),
            "indefinite" => Some(ContractType::Indefinite),
            "hourly" => Some(ContractType::Hourly),
            "services" => Some(ContractType::Services),
            _ => None,
        }
    }
}

/// AFP/Pension fund options (Colombian)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AFP {
    Colpensiones, // Fondo público
    Protección,
    #[serde(rename = "old_mutual")]
    OldMutual,
    Porvenir,
    Skandia,
    BBVA,
}

impl AFP {
    pub fn as_str(&self) -> &'static str {
        match self {
            AFP::Colpensiones => "colpensiones",
            AFP::Protección => "proteccion",
            AFP::OldMutual => "old_mutual",
            AFP::Porvenir => "porvenir",
            AFP::Skandia => "skandia",
            AFP::BBVA => "bbva",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "colpensiones" => Some(AFP::Colpensiones),
            "proteccion" => Some(AFP::Protección),
            "old_mutual" | "oldmutual" => Some(AFP::OldMutual),
            "porvenir" => Some(AFP::Porvenir),
            "skandia" => Some(AFP::Skandia),
            "bbva" => Some(AFP::BBVA),
            _ => None,
        }
    }

    /// AFP contribution rates (employee portion: 4%)
    pub fn rate(&self) -> f64 {
        0.04 // 4% employee contribution
    }
}

/// Bank account types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Savings,  // Cuenta de ahorro
    Checking, // Cuenta corriente
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Savings => "savings",
            AccountType::Checking => "checking",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "savings" => Some(AccountType::Savings),
            "checking" => Some(AccountType::Checking),
            _ => None,
        }
    }
}

/// Employee status in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmployeeStatus {
    Active,
    Inactive,
    Terminated,
}

impl EmployeeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmployeeStatus::Active => "active",
            EmployeeStatus::Inactive => "inactive",
            EmployeeStatus::Terminated => "terminated",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "active" => Some(EmployeeStatus::Active),
            "inactive" => Some(EmployeeStatus::Inactive),
            "terminated" => Some(EmployeeStatus::Terminated),
            _ => None,
        }
    }
}

/// Employee entity - core domain model for payroll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: String,
    pub user_id: Option<String>,
    pub document_type: DocumentType,
    pub document_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: String,
    pub department: String,
    pub contract_type: ContractType,
    pub base_salary: f64,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub account_type: Option<AccountType>,
    pub cci: Option<String>, // Código Interbancario
    pub afp: Option<AFP>,
    pub hire_date: DateTime<Utc>,
    pub termination_date: Option<DateTime<Utc>>,
    pub status: EmployeeStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Employee {
    /// Create a new employee
    pub fn new(
        id: String,
        document_type: DocumentType,
        document_number: String,
        first_name: String,
        last_name: String,
        email: String,
        position: String,
        department: String,
        contract_type: ContractType,
        base_salary: f64,
        hire_date: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            user_id: None,
            document_type,
            document_number,
            first_name,
            last_name,
            email,
            phone: None,
            address: None,
            position,
            department,
            contract_type,
            base_salary,
            bank_name: None,
            bank_account: None,
            account_type: None,
            cci: None,
            afp: None,
            hire_date,
            termination_date: None,
            status: EmployeeStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Update employee details
    pub fn update(
        &mut self,
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        address: Option<String>,
    ) {
        if let Some(v) = first_name {
            self.first_name = v;
        }
        if let Some(v) = last_name {
            self.last_name = v;
        }
        if let Some(v) = email {
            self.email = v;
        }
        if let Some(v) = phone {
            self.phone = Some(v);
        }
        if let Some(v) = address {
            self.address = Some(v);
        }
        self.updated_at = Utc::now();
    }

    /// Update position and department
    pub fn update_position(&mut self, position: String, department: String) {
        self.position = position;
        self.department = department;
        self.updated_at = Utc::now();
    }

    /// Update salary
    pub fn update_salary(&mut self, base_salary: f64) {
        self.base_salary = base_salary;
        self.updated_at = Utc::now();
    }

    /// Update bank information
    pub fn update_bank_info(
        &mut self,
        bank_name: Option<String>,
        bank_account: Option<String>,
        account_type: Option<AccountType>,
        cci: Option<String>,
    ) {
        self.bank_name = bank_name;
        self.bank_account = bank_account;
        self.account_type = account_type;
        self.cci = cci;
        self.updated_at = Utc::now();
    }

    /// Update AFP
    pub fn update_afp(&mut self, afp: AFP) {
        self.afp = Some(afp);
        self.updated_at = Utc::now();
    }

    /// Terminate employee
    pub fn terminate(&mut self, termination_date: DateTime<Utc>) {
        self.termination_date = Some(termination_date);
        self.status = EmployeeStatus::Terminated;
        self.updated_at = Utc::now();
    }

    /// Deactivate employee (soft delete)
    pub fn deactivate(&mut self) {
        self.status = EmployeeStatus::Inactive;
        self.updated_at = Utc::now();
    }

    /// Check if employee is active
    pub fn is_active(&self) -> bool {
        self.status == EmployeeStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_creation() {
        let employee = Employee::new(
            "test-id".to_string(),
            DocumentType::DNI,
            "12345678".to_string(),
            "Juan".to_string(),
            "Pérez".to_string(),
            "juan@example.com".to_string(),
            "Profesor".to_string(),
            "Academico".to_string(),
            ContractType::Indefinite,
            3000.0,
            Utc::now(),
        );

        assert_eq!(employee.first_name, "Juan");
        assert_eq!(employee.last_name, "Pérez");
        assert_eq!(employee.base_salary, 3000.0);
        assert!(employee.is_active());
    }

    #[test]
    fn test_full_name() {
        let employee = Employee::new(
            "test-id".to_string(),
            DocumentType::DNI,
            "12345678".to_string(),
            "Juan".to_string(),
            "Pérez".to_string(),
            "juan@example.com".to_string(),
            "Profesor".to_string(),
            "Academico".to_string(),
            ContractType::Indefinite,
            3000.0,
            Utc::now(),
        );

        assert_eq!(employee.full_name(), "Juan Pérez");
    }

    #[test]
    fn test_afp_rates() {
        assert_eq!(AFP::Prima.rates(), (10.00, 1.25));
        assert_eq!(AFP::Habitat.rates(), (10.00, 1.35));
        assert_eq!(AFP::Integra.rates(), (10.00, 1.45));
        assert_eq!(AFP::Profuturo.rates(), (10.00, 1.60));
        assert_eq!(AFP::ONP.rates(), (13.00, 0.0));
    }

    #[test]
    fn test_terminate_employee() {
        let mut employee = Employee::new(
            "test-id".to_string(),
            DocumentType::DNI,
            "12345678".to_string(),
            "Juan".to_string(),
            "Pérez".to_string(),
            "juan@example.com".to_string(),
            "Profesor".to_string(),
            "Academico".to_string(),
            ContractType::Indefinite,
            3000.0,
            Utc::now(),
        );

        employee.terminate(Utc::now());

        assert_eq!(employee.status, EmployeeStatus::Terminated);
        assert!(employee.termination_date.is_some());
    }
}
