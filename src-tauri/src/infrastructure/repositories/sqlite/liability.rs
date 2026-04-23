//! Liability and Equity SQLite Repositories
//!
//! Persist liabilities (debts/pasivos) and equity (patrimonio) to SQLite

use crate::infrastructure::database::SqlitePool;
use std::sync::Arc;

/// Liability entity
#[derive(Debug, Clone)]
pub struct Liability {
    pub id: String,
    pub provider_name: String,
    pub document_type: String,
    pub document_number: String,
    pub amount: f64,
    pub paid_amount: f64,
    pub liability_type: String,
    pub due_date: String,
    pub status: String,
    pub description: Option<String>,
    pub account_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Equity entity
#[derive(Debug, Clone)]
pub struct Equity {
    pub id: String,
    pub equity_type: String,
    pub description: String,
    pub amount: f64,
    pub account_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// SQLite repository for liabilities
#[derive(Clone)]
pub struct SqliteLiabilityRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteLiabilityRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub fn create(&self, liability: &Liability) -> Result<(), String> {
        eprintln!("[LIABILITY REPO] Creating liability: {} - S/{}", liability.provider_name, liability.amount);
        eprintln!("[LIABILITY REPO] Pool path: {:?}", self.pool.path());
        
        let sql = r#"
            INSERT INTO liabilities (
                id, provider_name, document_type, document_number, amount, paid_amount,
                liability_type, due_date, status, description, account_code, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let amount_str = liability.amount.to_string();
        let paid_amount_str = liability.paid_amount.to_string();
        let description_str = liability.description.clone().unwrap_or_default();
        let account_code_str = liability.account_code.clone().unwrap_or_default();

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        
        eprintln!("[LIABILITY REPO] Running INSERT SQL...");
        
        let result = conn.execute(
            sql,
            &[
                &liability.id as &dyn rusqlite::ToSql,
                &liability.provider_name,
                &liability.document_type,
                &liability.document_number,
                &amount_str as &dyn rusqlite::ToSql,
                &paid_amount_str as &dyn rusqlite::ToSql,
                &liability.liability_type,
                &liability.due_date,
                &liability.status,
                &description_str as &dyn rusqlite::ToSql,
                &account_code_str as &dyn rusqlite::ToSql,
                &liability.created_at,
                &liability.updated_at,
            ],
        );
        
        match result {
            Ok(rows) => eprintln!("[LIABILITY REPO] INSERT successful, {} rows affected", rows),
            Err(ref e) => eprintln!("[LIABILITY REPO] INSERT FAILED: {}", e),
        }
        
        result.map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Liability>, String> {
        eprintln!("[LIABILITY REPO] list() - Pool path: {:?}", self.pool.path());
        
        let sql = r#"
            SELECT id, provider_name, document_type, document_number, amount, paid_amount,
                   liability_type, due_date, status, description, account_code, created_at, updated_at
            FROM liabilities ORDER BY due_date ASC
        "#;

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        
        let liabilities: Vec<Liability> = stmt
            .query_map([], |row| {
                let status: String = row.get(8)?;
                eprintln!("[LIABILITY REPO] Row - id: {}, status: '{}'", row.get::<_, String>(0)?, status);
                
                // amount and paid_amount are REAL in DB, get as f64 directly
                let amount: f64 = row.get(4)?;
                let paid_amount: f64 = row.get(5)?;
                
                Ok(Liability {
                    id: row.get(0)?,
                    provider_name: row.get(1)?,
                    document_type: row.get(2)?,
                    document_number: row.get(3)?,
                    amount,
                    paid_amount,
                    liability_type: row.get(6)?,
                    due_date: row.get(7)?,
                    status,
                    description: row.get(9)?,
                    account_code: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .map_err(|e| {
                eprintln!("[LIABILITY REPO] query_map error: {}", e);
                e.to_string()
            })?
            .filter_map(|r| {
                match r {
                    Ok(liab) => Some(liab),
                    Err(e) => {
                        eprintln!("[LIABILITY REPO] row error: {}", e);
                        None
                    }
                }
            })
            .collect();

        eprintln!("[LIABILITY REPO] Returning {} liabilities", liabilities.len());
        Ok(liabilities)
    }

    pub fn get_total_by_type(&self, liability_type: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount - paid_amount), 0) FROM liabilities WHERE liability_type = ? AND status != 'paid'";
        
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        
let total: f64 = conn
            .query_row(sql, [], |row| {
                row.get(0)
            })
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    pub fn get_total(&self) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount - paid_amount), 0) FROM liabilities WHERE status != 'paid'";
        
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        
        let total: f64 = conn
            .query_row(sql, [], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| e.to_string())?
            .parse()
            .unwrap_or(0.0);

        Ok(total)
    }
}

/// SQLite repository for equity
#[derive(Clone)]
pub struct SqliteEquityRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteEquityRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub fn create(&self, equity: &Equity) -> Result<(), String> {
        let sql = r#"
            INSERT INTO equities (
                id, equity_type, description, amount, account_code, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        let amount_str = equity.amount.to_string();
        let account_code_str = equity.account_code.clone().unwrap_or_default();

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        conn.execute(
            sql,
            &[
                &equity.id as &dyn rusqlite::ToSql,
                &equity.equity_type,
                &equity.description,
                &amount_str as &dyn rusqlite::ToSql,
                &account_code_str as &dyn rusqlite::ToSql,
                &equity.created_at,
                &equity.updated_at,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Equity>, String> {
        let sql = r#"
            SELECT id, equity_type, description, amount, account_code, created_at, updated_at
            FROM equities ORDER BY created_at DESC
        "#;

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        
        let equities = stmt
            .query_map([], |row| {
                // amount is REAL in DB, get as f64 directly
                let amount: f64 = row.get(3)?;
                Ok(Equity {
                    id: row.get(0)?,
                    equity_type: row.get(1)?,
                    description: row.get(2)?,
                    amount,
                    account_code: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect();

        Ok(equities)
    }

    pub fn get_total_by_type(&self, equity_type: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM equities WHERE equity_type = ?";
        
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        
        let total: f64 = conn
            .query_row(sql, [equity_type], |row| {
                row.get(0)
            })
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    pub fn get_total(&self) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM equities";
        
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        
        let total: f64 = conn
            .query_row(sql, [], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| e.to_string())?
            .parse()
            .unwrap_or(0.0);

        Ok(total)
    }
}