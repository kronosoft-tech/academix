//! Accounting Service
//!
//! Use case for accounting operations - general ledger, trial balance, income statement.

use crate::application::dto::accounting::{
    AccountBalanceDto, AccountCategoryDto, AccountCategoryTreeNode, AccountingEntryDto,
    AccountingSummary, CategoryTotalDto, CreateEntryRequest, ExpenseByCategory,
    FinancialBalanceDto, IncomeStatementDto, MonthlyDataPoint, TrialBalanceAccountDto,
    TrialBalanceDto,
};
use crate::application::ports::accounting::{AccountCategoryRepository, AccountingEntryRepository};
use crate::domain::entities::accounting::{AccountingEntry, CategoryType, EntryType};
use crate::infrastructure::repositories::liability::{SqliteLiabilityRepository, SqliteEquityRepository};
use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

/// Accounting service - orchestrates accounting operations
/// Uses concrete repository types for liability/equity tables access
pub struct AccountingService<R: AccountingEntryRepository, C: AccountCategoryRepository> {
    entry_repo: R,
    category_repo: C,
    liability_repo: SqliteLiabilityRepository,
    equity_repo: SqliteEquityRepository,
}

impl<R: AccountingEntryRepository, C: AccountCategoryRepository> AccountingService<R, C> {
    pub fn new(entry_repo: R, category_repo: C, liability_repo: SqliteLiabilityRepository, equity_repo: SqliteEquityRepository) -> Self {
        Self {
            entry_repo,
            category_repo,
            liability_repo,
            equity_repo,
        }
    }

    /// Get account by code (e.g., "2105", "3105")
    pub fn get_account_by_code(&self, code: &str) -> Result<Option<AccountCategoryDto>, String> {
        let account = self.category_repo.get_by_code(code)?;
        Ok(account.map(AccountCategoryDto::from))
    }

    /// Create a new accounting entry
    pub fn create_entry(
        &self,
        request: CreateEntryRequest,
        created_by: String,
    ) -> Result<AccountingEntryDto, String> {
        // Validate: debit != credit
        if request.debit_account == request.credit_account {
            return Err("Debit and credit accounts must be different".to_string());
        }
        // Validate: amount > 0
        if request.amount <= 0.0 {
            return Err("Amount must be greater than 0".to_string());
        }

        // Verify accounts exist - try by ID first, then by code
        let debit_acc = self
            .category_repo
            .get_by_id(&request.debit_account)
            .ok()
            .flatten()
            .or_else(|| {
                self.category_repo
                    .get_by_code(&request.debit_account)
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| format!("Debit account not found: {}", request.debit_account))?;
        let credit_acc = self
            .category_repo
            .get_by_id(&request.credit_account)
            .ok()
            .flatten()
            .or_else(|| {
                self.category_repo
                    .get_by_code(&request.credit_account)
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| format!("Credit account not found: {}", request.credit_account))?;

        // Parse date - support both YYYY-MM-DD and RFC3339
        let date = if request.date.contains('T') {
            DateTime::parse_from_rfc3339(&request.date)
                .map_err(|e| format!("Invalid date: {}", e))?
                .with_timezone(&Utc)
        } else {
            // Parse YYYY-MM-DD format
            let parsed = chrono::NaiveDate::parse_from_str(&request.date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid date format: {}", e))?;
            // Create datetime from date parts
            let datetime =
                NaiveDateTime::new(parsed, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc)
        };

        // Generate reference if not provided
        let reference = request.reference.unwrap_or_else(|| {
            self.entry_repo
                .get_next_reference("AS")
                .map(|n| format!("AS-{:04}", n))
                .unwrap_or_else(|_| "AS-0001".to_string())
        });

        // Generate UUID v7 for the entry
        let ts = uuid::Timestamp::now(uuid::NoContext);
        let entry_id = Uuid::new_v7(ts).to_string();

        let entry = AccountingEntry::new(
            entry_id,
            date,
            reference,
            request.description,
            request.debit_account.clone(),
            request.credit_account.clone(),
            request.amount,
            created_by,
        );

        let created = self.entry_repo.create(entry)?;

        // Update account balances
        // Debit increases asset/expense/cost, decreases liability/equity/income
        // Credit increases liability/equity/income, decreases asset/expense/cost
        self.update_account_balance(&request.debit_account, request.amount, true)?;
        self.update_account_balance(&request.credit_account, request.amount, false)?;

        // Build DTO with account names
        let mut dto = AccountingEntryDto::from(created);
        dto.debit_account_name = debit_acc.name;
        dto.credit_account_name = credit_acc.name;

        Ok(dto)
    }

    /// Update account balance after entry
    fn update_account_balance(
        &self,
        account_id: &str,
        amount: f64,
        is_debit: bool,
    ) -> Result<(), String> {
        let category = self
            .category_repo
            .get_by_id(account_id)?
            .ok_or_else(|| format!("Account not found: {}", account_id))?;

        let balance_change = if category.category_type.is_debit_increase() {
            if is_debit {
                amount
            } else {
                -amount
            }
        } else {
            if is_debit {
                -amount
            } else {
                amount
            }
        };

        self.category_repo
            .update_balance(account_id, balance_change)
    }

    /// Get accounting entry by ID
    pub fn get_entry(&self, id: &str) -> Result<Option<AccountingEntryDto>, String> {
        let entry = self.entry_repo.get_by_id(id)?;

        if let Some(entry) = entry {
            let debit_acc = self.category_repo.get_by_id(&entry.debit_account)?;
            let credit_acc = self.category_repo.get_by_id(&entry.credit_account)?;

            let mut dto = AccountingEntryDto::from(entry);
            if let Some(acc) = debit_acc {
                dto.debit_account_name = acc.name;
            }
            if let Some(acc) = credit_acc {
                dto.credit_account_name = acc.name;
            }
            Ok(Some(dto))
        } else {
            Ok(None)
        }
    }

    /// List accounting entries with filters
    pub fn list_entries(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntryDto>, String> {
        let entries = self.entry_repo.list(date_from, date_to, entry_type)?;

        let mut dtos = Vec::new();
        for entry in entries {
            let debit_acc = self.category_repo.get_by_id(&entry.debit_account)?;
            let credit_acc = self.category_repo.get_by_id(&entry.credit_account)?;

            let mut dto = AccountingEntryDto::from(entry);
            if let Some(acc) = debit_acc {
                dto.debit_account_name = acc.name;
            }
            if let Some(acc) = credit_acc {
                dto.credit_account_name = acc.name;
            }
            dtos.push(dto);
        }

        Ok(dtos)
    }

    /// Get trial balance (balance de comprobación)
    pub fn get_trial_balance(&self, _as_of_date: &str) -> Result<TrialBalanceDto, String> {
        // Get all active accounts
        let accounts = self.category_repo.list(None, true)?;

        let mut account_dtos = Vec::new();
        let mut total_debits = 0.0;
        let mut total_credits = 0.0;

        for account in accounts {
            let (debit, credit, balance_type) = if account.category_type.is_debit_increase() {
                if account.balance >= 0.0 {
                    (account.balance, 0.0, "debit")
                } else {
                    (0.0, account.balance.abs(), "credit")
                }
            } else {
                if account.balance >= 0.0 {
                    (0.0, account.balance, "credit")
                } else {
                    (account.balance.abs(), 0.0, "debit")
                }
            };

            if debit > 0.0 || credit > 0.0 {
                account_dtos.push(TrialBalanceAccountDto {
                    account_id: account.id.clone(),
                    account_code: account.code.clone(),
                    account_name: account.name.clone(),
                    debit_balance: debit,
                    credit_balance: credit,
                    balance_type: balance_type.to_string(),
                });
                total_debits += debit;
                total_credits += credit;
            }
        }

        // Sort by account code
        account_dtos.sort_by(|a, b| a.account_code.cmp(&b.account_code));

        Ok(TrialBalanceDto {
            as_of_date: Utc::now().to_rfc3339(),
            accounts: account_dtos,
            total_debits,
            total_credits,
            is_balanced: (total_debits - total_credits).abs() < 0.01,
        })
    }

    /// Get income statement (estado de resultados)
    pub fn get_income_statement(
        &self,
        period_start: &str,
        period_end: &str,
    ) -> Result<IncomeStatementDto, String> {
        // Get income and expense accounts
        let income_accounts = self
            .category_repo
            .get_balances_by_type(CategoryType::Income)?;
        let expense_accounts = self
            .category_repo
            .get_balances_by_type(CategoryType::Expense)?;
        let cost_accounts = self
            .category_repo
            .get_balances_by_type(CategoryType::Cost)?;

        let total_income: f64 = income_accounts.iter().map(|a| a.balance).sum();
        let total_expenses: f64 = expense_accounts.iter().map(|a| a.balance).sum();
        let total_costs: f64 = cost_accounts.iter().map(|a| a.balance).sum();

        let income_by_category: Vec<CategoryTotalDto> = income_accounts
            .into_iter()
            .map(|a| CategoryTotalDto {
                category_id: a.id,
                category_name: a.name,
                total: a.balance,
            })
            .collect();

        let expenses_by_category: Vec<CategoryTotalDto> = expense_accounts
            .into_iter()
            .chain(cost_accounts.into_iter())
            .map(|a| CategoryTotalDto {
                category_id: a.id,
                category_name: a.name,
                total: a.balance,
            })
            .collect();

        let net_result = total_income - total_expenses - total_costs;

        Ok(IncomeStatementDto {
            period_start: period_start.to_string(),
            period_end: period_end.to_string(),
            total_income,
            total_expenses,
            total_costs,
            net_result,
            is_profitable: net_result > 0.0,
            income_by_category,
            expenses_by_category,
        })
    }

    /// List account categories (chart of accounts)
    pub fn list_accounts(
        &self,
        category_type: Option<CategoryType>,
        active_only: bool,
    ) -> Result<Vec<AccountCategoryDto>, String> {
        let accounts = self.category_repo.list(category_type, active_only)?;
        Ok(accounts.into_iter().map(AccountCategoryDto::from).collect())
    }

    /// Get account categories as tree
    pub fn get_account_tree(&self) -> Result<Vec<AccountCategoryTreeNode>, String> {
        let root_accounts = self.category_repo.list_roots()?;

        let mut tree = Vec::new();
        for root in root_accounts {
            let children = self.get_children(&root.id)?;
            tree.push(AccountCategoryTreeNode {
                category: AccountCategoryDto::from(root),
                children,
            });
        }

        Ok(tree)
    }

    fn get_children(&self, parent_id: &str) -> Result<Vec<AccountCategoryTreeNode>, String> {
        let children = self.category_repo.list_children(parent_id)?;

        let mut result = Vec::new();
        for child in children {
            let grandchildren = self.get_children(&child.id)?;
            result.push(AccountCategoryTreeNode {
                category: AccountCategoryDto::from(child),
                children: grandchildren,
            });
        }

        Ok(result)
    }

    /// Get accounting summary for dashboard
    pub fn get_summary(&self) -> Result<AccountingSummary, String> {
        let entries = self.entry_repo.list(None, None, None)?;
        let accounts = self.category_repo.list(None, true)?;

        // Build account lookup map
        let account_map: std::collections::HashMap<String, String> = accounts
            .iter()
            .map(|a| (a.code.clone(), a.name.clone()))
            .collect();

        // Calculate income and expenses
        let mut total_income: f64 = 0.0;
        let mut total_expenses: f64 = 0.0;
        let mut expenses_by_cat: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut income_by_cat: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();

        // For monthly data (last 6 months)
        let months = [
            "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
        ];
        let mut monthly_income: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut monthly_expenses: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();

        for entry in &entries {
            // Get month from date
            let month_name = entry.date.format("%m").to_string();
            let month_idx: usize = month_name.parse().unwrap_or(1) - 1;
            let month_label = months.get(month_idx).unwrap_or(&"???");

            // Check credit account (income)
            if entry.credit_account.starts_with('6') {
                total_income += entry.amount;
                *monthly_income.entry(month_label.to_string()).or_insert(0.0) += entry.amount;

                // Track by category
                let cat_name = account_map
                    .get(&entry.credit_account)
                    .cloned()
                    .unwrap_or_else(|| entry.credit_account.clone());
                *income_by_cat.entry(cat_name).or_insert(0.0) += entry.amount;
            }
            // Check debit account (expenses)
            if entry.debit_account.starts_with('4') || entry.debit_account.starts_with('5') {
                total_expenses += entry.amount;
                *monthly_expenses
                    .entry(month_label.to_string())
                    .or_insert(0.0) += entry.amount;

                // Track by category
                let cat_name = account_map
                    .get(&entry.debit_account)
                    .cloned()
                    .unwrap_or_else(|| entry.debit_account.clone());
                *expenses_by_cat.entry(cat_name).or_insert(0.0) += entry.amount;
            }
        }

        let balance = total_income - total_expenses;
        let entry_count = entries.len() as i64;

        // Build monthly data (last 6 months)
        let now = chrono::Utc::now();
        let monthly_data: Vec<MonthlyDataPoint> = (0..6)
            .rev()
            .map(|i| {
                let date = now - chrono::Duration::days(30 * i);
                let idx = date.format("%m").to_string().parse::<usize>().unwrap_or(1) - 1;
                let m = months.get(idx).unwrap_or(&"???").to_string();
                MonthlyDataPoint {
                    month: m.clone(),
                    income: *monthly_income.get(&m).unwrap_or(&0.0),
                    expenses: *monthly_expenses.get(&m).unwrap_or(&0.0),
                }
            })
            .collect();

        // Build expense breakdown (top 5)
        let mut expenses_vec: Vec<(String, f64)> = expenses_by_cat.into_iter().collect();
        expenses_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let expenses_by_category: Vec<ExpenseByCategory> = expenses_vec
            .into_iter()
            .take(5)
            .map(|(name, amount)| ExpenseByCategory {
                category_name: name,
                amount,
            })
            .collect();

        // Build income breakdown (top 5)
        let mut income_vec: Vec<(String, f64)> = income_by_cat.into_iter().collect();
        income_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let income_by_category: Vec<ExpenseByCategory> = income_vec
            .into_iter()
            .take(5)
            .map(|(name, amount)| ExpenseByCategory {
                category_name: name,
                amount,
            })
            .collect();

        // Get recent entries (last 10)
        let recent: Vec<AccountingEntryDto> = entries
            .into_iter()
            .take(10)
            .map(|e| AccountingEntryDto::from(e))
            .collect();

        Ok(AccountingSummary {
            total_income,
            total_expenses,
            net_balance: balance,
            account_count: accounts.len() as i64,
            entry_count,
            recent_entries: recent,
            monthly_data,
            expenses_by_category,
            income_by_category,
        })
    }

    /// Get financial balance (Balance Financiero) - Opción B con toque de C
    /// 
    /// Estructura según contabilidad estándar:
    /// - Activos (1xxx): Bienes y derechos
    /// - Pasivos (2xxx + tabla): Deudas separadas por corto/largo plazo
    /// - Patrimonio: Capital + Reservas + Resultados Acumulados + Resultado del Ejercicio
    pub fn get_financial_balance(&self, as_of_date: &str) -> Result<FinancialBalanceDto, String> {
        // Get all accounts with their balances
        let all_accounts = self.category_repo.list(None, true)?;

        // Separate by account type (first digit)
        let mut assets: Vec<AccountBalanceDto> = Vec::new();
        let mut liabilities_current: Vec<AccountBalanceDto> = Vec::new(); // Corto plazo
        let mut liabilities_long: Vec<AccountBalanceDto> = Vec::new();    // Largo plazo
        let mut equity_capital: Vec<AccountBalanceDto> = Vec::new();    // 31xx
        let mut equity_reserves: Vec<AccountBalanceDto> = Vec::new(); // 32xx
        let mut equity_retained: Vec<AccountBalanceDto> = Vec::new(); // 37xx (Resultados acumulados)
        let mut income: Vec<AccountBalanceDto> = Vec::new();      // 6xxx
        let mut expenses: Vec<AccountBalanceDto> = Vec::new();   // 4xxx, 5xxx, 7xxx
        
        let mut total_assets: f64 = 0.0;
        let mut total_liabilities_current: f64 = 0.0;
        let mut total_liabilities_long: f64 = 0.0;
        let mut total_equity_capital: f64 = 0.0;
        let mut total_equity_reserves: f64 = 0.0;
        let mut total_equity_retained: f64 = 0.0;
        let mut total_income: f64 = 0.0;
        let mut total_expenses: f64 = 0.0;

        for acc in &all_accounts {
            let balance = acc.balance;
            if balance == 0.0 {
                continue;
            }

            let code = &acc.code;
            let first_digit = code.chars().next().unwrap_or('0');

            match first_digit {
                '1' => {
                    // Activos (cuentas 1xxx)
                    total_assets += balance;
                    assets.push(AccountBalanceDto {
                        account_code: acc.code.clone(),
                        account_name: acc.name.clone(),
                        balance,
                    });
                }
                '2' => {
                    // Pasivos (cuentas 2xxx) - Separar por tipo de cuenta
                    // Cuentas 21xx-2199: Corto Plazo (proveedores, Impuesto, nóminas)
                    // Cuentas 22xx+: Largo Plazo (préstamos, hipotecas)
                    if code.starts_with("21") || code.starts_with("22") {
                        // Clasificación básica
                        if code.starts_with("21") {
                            total_liabilities_current += balance;
                            liabilities_current.push(AccountBalanceDto {
                                account_code: acc.code.clone(),
                                account_name: acc.name.clone(),
                                balance,
                            });
                        } else {
                            total_liabilities_long += balance;
                            liabilities_long.push(AccountBalanceDto {
                                account_code: acc.code.clone(),
                                account_name: acc.name.clone(),
                                balance,
                            });
                        }
                    } else {
                        // Otras cuentas de pasivo van a largo plazo por defecto
                        total_liabilities_long += balance;
                        liabilities_long.push(AccountBalanceDto {
                            account_code: acc.code.clone(),
                            account_name: acc.name.clone(),
                            balance,
                        });
                    }
                }
                '3' => {
                    // Patrimonio (cuentas 3xxx)
                    // 31xx: Capital Social
                    // 32xx: Reservas
                    // 33xx-36xx: Resultados (ejercicio actual -> se calcula)
                    // 37xx: Resultados acumulados
                    if code.starts_with("31") {
                        total_equity_capital += balance;
                        equity_capital.push(AccountBalanceDto {
                            account_code: acc.code.clone(),
                            account_name: acc.name.clone(),
                            balance,
                        });
                    } else if code.starts_with("32") {
                        total_equity_reserves += balance;
                        equity_reserves.push(AccountBalanceDto {
                            account_code: acc.code.clone(),
                            account_name: acc.name.clone(),
                            balance,
                        });
                    } else if code.starts_with("37") {
                        total_equity_retained += balance;
                        equity_retained.push(AccountBalanceDto {
                            account_code: acc.code.clone(),
                            account_name: acc.name.clone(),
                            balance,
                        });
                    } else {
                        // 33xx-36xx: résultat del ejercicio (se sumando pero no se cuenta doble al final)
                        // Se incluye en el cálculo de resultado
                    }
                }
                '4' | '5' | '7' => {
                    // Gastos (4xxx: Gastos, 5xxx: Costo de venta, 7xxx: Otros gastos)
                    total_expenses += balance;
                    expenses.push(AccountBalanceDto {
                        account_code: acc.code.clone(),
                        account_name: acc.name.clone(),
                        balance,
                    });
                }
                '6' => {
                    // Ingresos (6xxx)
                    total_income += balance;
                    income.push(AccountBalanceDto {
                        account_code: acc.code.clone(),
                        account_name: acc.name.clone(),
                        balance,
                    });
                }
                _ => {}
            }
        }

        // Add liabilities from liabilities table (pasivos registrados manualmente)
        // Separar por tipo: short_term -> Corto Plazo, long_term -> Largo Plazo
        println!("[DEBUG] Querying liabilities table...");
        if let Ok(liabilities_list) = self.liability_repo.list() {
            println!("[DEBUG] Found {} liabilities", liabilities_list.len());
            for liab in liabilities_list.iter() {
                let pending = liab.amount - liab.paid_amount;
                if pending > 0.0 {
                    let account_code = liab.account_code.clone().unwrap_or_else(|| {
                        if liab.liability_type == "short_term" { "2105".to_string() } 
                        else { "2205".to_string() }
                    });
                    
                    let dto = AccountBalanceDto {
                        account_code: account_code.clone(),
                        account_name: format!("[{}] {} - {}", liab.liability_type, liab.provider_name, liab.description.clone().unwrap_or_default()),
                        balance: pending,
                    };
                    
                    if liab.liability_type == "short_term" {
                        total_liabilities_current += pending;
                        liabilities_current.push(dto);
                    } else {
                        total_liabilities_long += pending;
                        liabilities_long.push(dto);
                    }
                }
            }
        }

        // Add equity from equities table (patrimonio registrado manualmente)
        println!("[DEBUG] Querying equities table...");
        if let Ok(equities_list) = self.equity_repo.list() {
            println!("[DEBUG] Found {} equities", equities_list.len());
            for eq in equities_list.iter() {
                let account_code = eq.account_code.clone().unwrap_or_else(|| "3105".to_string());
                
                let dto = AccountBalanceDto {
                    account_code: account_code.clone(),
                    account_name: format!("[{}] {}", eq.equity_type, eq.description),
                    balance: eq.amount,
                };
                
                // Separar por tipo de equity
                match eq.equity_type.as_str() {
                    "capital" => {
                        total_equity_capital += eq.amount;
                        equity_capital.push(dto);
                    }
                    "reserves" => {
                        total_equity_reserves += eq.amount;
                        equity_reserves.push(dto);
                    }
                    "results" | "retained" => {
                        total_equity_retained += eq.amount;
                        equity_retained.push(dto);
                    }
                    _ => {}
                }
            }
        }

        // Sort each category by balance descending
        assets.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        liabilities_current.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        liabilities_long.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        equity_capital.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        equity_reserves.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        equity_retained.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        income.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));
        expenses.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate Resultado del Ejercicio = Ingresos - Gastos
        let net_result = total_income - total_expenses;

        // Total Pasivos = Corto Plazo + Largo Plazo
        let total_liabilities = total_liabilities_current + total_liabilities_long;

        // Total Patrimonio = Capital + Reservas + Resultados Acumulados
        // IMPORTANTE: NO sumar net_result aquí porque ya está implícito en los Activos
        // (las ventas aumentan Caja, los gastos la reducen - el resultado ya está reflejado)
        let total_equity = total_equity_capital + total_equity_reserves + total_equity_retained;

        // DEBUG: Mostrar desglose para entender por qué no cuadra
        eprintln!("[BALANCE DEBUG]");
        eprintln!("  Activos (1xxx): S/ {:.2}", total_assets);
        eprintln!("  Pasivos CP (21xx): S/ {:.2}", total_liabilities_current);
        eprintln!("  Pasivos LP (22xx+): S/ {:.2}", total_liabilities_long);
        eprintln!("  Total Pasivos: S/ {:.2}", total_liabilities);
        eprintln!("  Capital (31xx): S/ {:.2}", total_equity_capital);
        eprintln!("  Reservas (32xx): S/ {:.2}", total_equity_reserves);
        eprintln!("  Retained (37xx): S/ {:.2}", total_equity_retained);
        eprintln!("  Resultado Ejercicio (solo informativo): S/ {:.2}", net_result);
        eprintln!("  Total Patrimonio: S/ {:.2}", total_equity);
        eprintln!("  =============================");
        eprintln!("  Activos: S/ {:.2}", total_assets);
        eprintln!("  Pasivos + Patrimonio: S/ {:.2}", total_liabilities + total_equity);
        eprintln!("  Diferencia: S/ {:.2}", total_assets - (total_liabilities + total_equity));

        // Verificar: Activos = Pasivos + Patrimonio
        let accounting_equation = total_liabilities + total_equity;
        let is_balanced = (total_assets - accounting_equation).abs() < 1.0;

        // Combinar para el DTO de retorno
        // Assets siempre en una lista, liabilities + equity juntos para el balance
        let mut all_liabilities: Vec<AccountBalanceDto> = liabilities_current.clone();
        all_liabilities.extend(liabilities_long.clone());
        
        let mut all_equity: Vec<AccountBalanceDto> = equity_capital.clone();
        all_equity.extend(equity_reserves.clone());
        all_equity.extend(equity_retained.clone());
        
        // Agregar resultado del ejercicio PARA INFORMACIÓN (no se suma al total)
        if net_result != 0.0 {
            all_equity.push(AccountBalanceDto {
                account_code: "3605".to_string(),
                account_name: format!("Resultado del Ejercicio (no incluido, ya en Activos): S/ {:.2}", net_result),
                balance: net_result,
            });
        }

        Ok(FinancialBalanceDto {
            as_of_date: as_of_date.to_string(),
            assets,
            liabilities: all_liabilities,
            equity: all_equity,
            total_assets,
            total_liabilities,
            total_equity,
            is_balanced,
        })
    }
}
