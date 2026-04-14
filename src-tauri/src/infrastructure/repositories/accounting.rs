//! Accounting Repository Ports (Trait Definitions)
//!
//! In-memory repository implementations for accounting.

use crate::application::ports::accounting::{AccountCategoryRepository, AccountingEntryRepository};
use crate::domain::entities::accounting::{
    AccountCategory, AccountingEntry, CategoryType, EntryType,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory implementation of AccountCategoryRepository
pub struct InMemoryAccountCategoryRepository {
    categories: Arc<RwLock<HashMap<String, AccountCategory>>>,
    next_id: Arc<RwLock<u32>>,
}

impl InMemoryAccountCategoryRepository {
    pub fn new() -> Self {
        let repo = Self {
            categories: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        };
        repo.seed_default_accounts();
        repo
    }

    fn seed_default_accounts(&self) {
        let default_accounts = vec![
            // 1xxx - Activos (Debe)
            ("caja", "1101", "Caja", CategoryType::Asset),
            ("banco", "1111", "Banco", CategoryType::Asset),
            (
                "cuentas_cobrar",
                "1201",
                "Cuentas por Cobrar",
                CategoryType::Asset,
            ),
            // 2xxx - Pasivos (Haber)
            (
                "proveedores",
                "2101",
                "Proveedores",
                CategoryType::Liability,
            ),
            (
                "remuneraciones",
                "2201",
                "Remuneraciones por Pagar",
                CategoryType::Liability,
            ),
            // 3xxx - Patrimonio
            ("capital", "3101", "Capital", CategoryType::Equity),
            ("reservas", "3201", "Reservas", CategoryType::Equity),
            // 4xxx - Gastos (Débito)
            (
                "sueldos",
                "4101",
                "Sueldos y Salarios",
                CategoryType::Expense,
            ),
            ("servicios", "4102", "Servicios", CategoryType::Expense),
            (
                "gastos_varios",
                "4103",
                "Gastos Varios",
                CategoryType::Expense,
            ),
            // 5xxx - Costos
            ("costos", "5101", "Costos de Servicios", CategoryType::Cost),
            // 6xxx - Ingresos (Crédito)
            (
                "servicios_edu",
                "6101",
                "Servicios Educativos",
                CategoryType::Income,
            ),
            ("matriculas", "6102", "Matrículas", CategoryType::Income),
            (
                "otros_ingresos",
                "6103",
                "Otros Ingresos",
                CategoryType::Income,
            ),
        ];

        let mut categories = self.categories.write().unwrap();
        for (id, code, name, cat_type) in default_accounts {
            let category =
                AccountCategory::new(id.to_string(), code.to_string(), name.to_string(), cat_type);
            categories.insert(id.to_string(), category);
        }
    }

    fn generate_id(&self) -> String {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("acc-{:03}", *counter);
        *counter += 1;
        id
    }
}

impl AccountCategoryRepository for InMemoryAccountCategoryRepository {
    fn create(&self, mut category: AccountCategory) -> Result<AccountCategory, String> {
        if category.id.is_empty() {
            category.id = self.generate_id();
        }
        let mut categories = self.categories.write().unwrap();
        categories.insert(category.id.clone(), category.clone());
        Ok(category)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<AccountCategory>, String> {
        let categories = self.categories.read().unwrap();
        Ok(categories.get(id).cloned())
    }

    fn get_by_code(&self, code: &str) -> Result<Option<AccountCategory>, String> {
        let categories = self.categories.read().unwrap();
        Ok(categories.values().find(|c| c.code == code).cloned())
    }

    fn list(
        &self,
        category_type: Option<CategoryType>,
        active_only: bool,
    ) -> Result<Vec<AccountCategory>, String> {
        let categories = self.categories.read().unwrap();
        let mut result: Vec<AccountCategory> = categories
            .values()
            .filter(|c| {
                let type_match = category_type.map_or(true, |t| c.category_type == t);
                let active_match = !active_only || c.active;
                type_match && active_match
            })
            .cloned()
            .collect();
        result.sort_by(|a, b| a.code.cmp(&b.code));
        Ok(result)
    }

    fn list_roots(&self) -> Result<Vec<AccountCategory>, String> {
        let categories = self.categories.read().unwrap();
        let mut result: Vec<AccountCategory> = categories
            .values()
            .filter(|c| c.parent_id.is_none())
            .cloned()
            .collect();
        result.sort_by(|a, b| a.code.cmp(&b.code));
        Ok(result)
    }

    fn list_children(&self, parent_id: &str) -> Result<Vec<AccountCategory>, String> {
        let categories = self.categories.read().unwrap();
        let parent = categories.get(parent_id).ok_or("Parent not found")?;
        let prefix = &parent.code[..3];

        let mut result: Vec<AccountCategory> = categories
            .values()
            .filter(|c| {
                c.parent_id.as_deref() == Some(parent_id)
                    || (c.code.starts_with(prefix) && c.code.len() > 3)
            })
            .cloned()
            .collect();
        result.sort_by(|a, b| a.code.cmp(&b.code));
        Ok(result)
    }

    fn update(&self, category: AccountCategory) -> Result<AccountCategory, String> {
        let mut categories = self.categories.write().unwrap();
        if categories.contains_key(&category.id) {
            categories.insert(category.id.clone(), category.clone());
            Ok(category)
        } else {
            Err(format!("Category not found: {}", category.id))
        }
    }

    fn update_balance(&self, id: &str, amount: f64) -> Result<(), String> {
        let mut categories = self.categories.write().unwrap();
        if let Some(category) = categories.get_mut(id) {
            category.update_balance(amount);
            Ok(())
        } else {
            Err(format!("Category not found: {}", id))
        }
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut categories = self.categories.write().unwrap();
        if let Some(category) = categories.get_mut(id) {
            category.deactivate();
            Ok(true)
        } else {
            Err(format!("Category not found: {}", id))
        }
    }

    fn get_balances_by_type(
        &self,
        category_type: CategoryType,
    ) -> Result<Vec<AccountCategory>, String> {
        self.list(Some(category_type), true)
    }
}

/// In-memory implementation of AccountingEntryRepository
pub struct InMemoryAccountingEntryRepository {
    entries: Arc<RwLock<HashMap<String, AccountingEntry>>>,
    next_id: Arc<RwLock<u32>>,
    next_ref: Arc<RwLock<u32>>,
}

impl InMemoryAccountingEntryRepository {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            next_ref: Arc::new(RwLock::new(1)),
        }
    }

    fn generate_id(&self) -> String {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("entry-{:03}", *counter);
        *counter += 1;
        id
    }

    pub fn generate_reference(&self, prefix: &str) -> String {
        let mut counter = self.next_ref.write().unwrap();
        let ref_num = *counter;
        *counter += 1;
        format!("{}-{:04}", prefix, ref_num)
    }
}

impl AccountingEntryRepository for InMemoryAccountingEntryRepository {
    fn create(&self, mut entry: AccountingEntry) -> Result<AccountingEntry, String> {
        if entry.id.is_empty() {
            entry.id = self.generate_id();
        }
        if entry.reference.is_empty() {
            entry.reference = self.generate_reference("AS");
        }
        let mut entries = self.entries.write().unwrap();
        entries.insert(entry.id.clone(), entry.clone());
        Ok(entry)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<AccountingEntry>, String> {
        let entries = self.entries.read().unwrap();
        Ok(entries.get(id).cloned())
    }

    fn list(
        &self,
        _date_from: Option<&str>,
        _date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntry>, String> {
        let entries = self.entries.read().unwrap();
        let mut result: Vec<AccountingEntry> = entries
            .values()
            .filter(|e| {
                let type_match = entry_type.map_or(true, |t| e.entry_type == t);
                type_match
            })
            .cloned()
            .collect();
        result.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(result)
    }

    fn get_by_related(
        &self,
        related_id: &str,
        related_type: &str,
    ) -> Result<Vec<AccountingEntry>, String> {
        let entries = self.entries.read().unwrap();
        let result: Vec<AccountingEntry> = entries
            .values()
            .filter(|e| {
                e.related_id.as_deref() == Some(related_id)
                    && e.related_type.as_deref() == Some(related_type)
            })
            .cloned()
            .collect();
        Ok(result)
    }

    fn get_by_account(&self, account_id: &str) -> Result<Vec<AccountingEntry>, String> {
        let entries = self.entries.read().unwrap();
        let result: Vec<AccountingEntry> = entries
            .values()
            .filter(|e| e.debit_account == account_id || e.credit_account == account_id)
            .cloned()
            .collect();
        Ok(result)
    }

    fn get_by_date_range(
        &self,
        _date_from: &str,
        _date_to: &str,
    ) -> Result<Vec<AccountingEntry>, String> {
        let entries = self.entries.read().unwrap();
        let mut result: Vec<AccountingEntry> = entries.values().cloned().collect();
        result.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(result)
    }

    fn update(&self, entry: AccountingEntry) -> Result<AccountingEntry, String> {
        let mut entries = self.entries.write().unwrap();
        if entries.contains_key(&entry.id) {
            entries.insert(entry.id.clone(), entry.clone());
            Ok(entry)
        } else {
            Err(format!("Entry not found: {}", entry.id))
        }
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut entries = self.entries.write().unwrap();
        if entries.remove(id).is_some() {
            Ok(true)
        } else {
            Err(format!("Entry not found: {}", id))
        }
    }

    fn get_total_debits(&self, _date_from: &str, _date_to: &str) -> Result<f64, String> {
        let entries = self.entries.read().unwrap();
        let total: f64 = entries.values().map(|e| e.amount).sum();
        Ok(total)
    }

    fn get_total_credits(&self, _date_from: &str, _date_to: &str) -> Result<f64, String> {
        let entries = self.entries.read().unwrap();
        let total: f64 = entries.values().map(|e| e.amount).sum();
        Ok(total)
    }

    fn get_next_reference(&self, _prefix: &str) -> Result<u32, String> {
        let counter = self.next_ref.read().unwrap();
        Ok(*counter)
    }
}
