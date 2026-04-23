//! Liability and Equity Commands - Pasivos y Patrimonio
//!
//! Commands for managing liabilities (debts) and equity (capital, reserves)

use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use crate::infrastructure::repositories::{
    SqliteLiabilityRepository, SqliteEquityRepository,
};
use crate::commands::accounting::AccountingServiceState;

/// Liability service state type
pub type LiabilityServiceState = SqliteLiabilityRepository;
pub type EquityServiceState = SqliteEquityRepository;

/// Create liability request (registrar deuda/pasivo)
#[derive(Debug, Deserialize)]
pub struct CreateLiabilityCommand {
    pub provider_name: String,
    pub document_type: String,
    pub document_number: String,
    pub amount: f64,
    pub liability_type: String,
    pub due_date: String,
    pub description: Option<String>,
}

/// Liability response (includes account_code for Balance Financiero)
#[derive(Debug, Serialize)]
pub struct LiabilityDto {
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

/// Create equity request (registrar patrimonio)
#[derive(Debug, Deserialize)]
pub struct CreateEquityCommand {
    pub equity_type: String,
    pub description: String,
    pub amount: f64,
    /// Account code where the money/asset went (e.g., "1105" for caja, "1110" for bancos)
    /// Required when registering capital to keep balance balanced
    pub asset_account_code: Option<String>,
}

/// Equity DTO (includes account_code for Balance Financiero)
#[derive(Debug, Serialize)]
pub struct EquityDto {
    pub id: String,
    pub equity_type: String,
    pub description: String,
    pub amount: f64,
    pub account_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create a new liability (registrar deuda)
#[tauri::command]
pub fn create_liability(
    state: State<LiabilityServiceState>,
    request: CreateLiabilityCommand,
) -> Result<LiabilityDto, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    // Determine account code based on liability type
    let account_code = match request.liability_type.as_str() {
        "short_term" => "2105",
        "long_term" => "2205",
        "provisions" => "2810",
        _ => "2105",
    };

    // Build entity using the repository's internal types
    use crate::infrastructure::repositories::liability::Liability;
    
    let entity = Liability {
        id: id.clone(),
        provider_name: request.provider_name.clone(),
        document_type: request.document_type.clone(),
        document_number: request.document_number.clone(),
        amount: request.amount,
        paid_amount: 0.0,
        liability_type: request.liability_type.clone(),
        due_date: request.due_date.clone(),
        status: "pending".to_string(),
        description: request.description.clone(),
        account_code: Some(account_code.to_string()),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    // Save to database using repository from state
    state.create(&entity)?;

    eprintln!("[DEBUG] Created liability: {} - S/ {} (account: {})", entity.provider_name, entity.amount, account_code);

    Ok(LiabilityDto {
        id,
        provider_name: request.provider_name,
        document_type: request.document_type,
        document_number: request.document_number,
        amount: request.amount,
        paid_amount: 0.0,
        liability_type: request.liability_type,
        due_date: request.due_date,
        status: "pending".to_string(),
        description: request.description,
        account_code: Some(account_code.to_string()),
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

/// List all liabilities
#[tauri::command]
pub fn list_liabilities(
    state: State<LiabilityServiceState>,
) -> Result<Vec<LiabilityDto>, String> {
    let liabilities = state.list()?;

    Ok(liabilities
        .into_iter()
        .map(|l| LiabilityDto {
            id: l.id,
            provider_name: l.provider_name,
            document_type: l.document_type,
            document_number: l.document_number,
            amount: l.amount,
            paid_amount: l.paid_amount,
            liability_type: l.liability_type,
            due_date: l.due_date,
            status: l.status,
            description: l.description,
            account_code: l.account_code,
            created_at: l.created_at,
            updated_at: l.updated_at,
        })
        .collect())
}

/// Pay a liability (registrar pago de deuda)
#[tauri::command]
pub fn pay_liability(
    _state: State<LiabilityServiceState>,
    id: String,
    amount: f64,
) -> Result<LiabilityDto, String> {
    // TODO: Implement actual payment
    eprintln!("[DEBUG] Paying liability {} amount: {}", id, amount);
    Err("Not implemented yet".to_string())
}

/// Create equity (registrar patrimonio)
/// IMPORTANT: When registering capital, automatically creates accounting entry
/// to keep the balance balanced: Debit (Caja/Bancos) / Credit (Capital)
#[tauri::command]
pub fn create_equity(
    equity_state: State<EquityServiceState>,
    accounting_state: State<AccountingServiceState>,
    request: CreateEquityCommand,
) -> Result<EquityDto, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    // Determine account code based on equity type
    let account_code = match request.equity_type.as_str() {
        "capital" => "3105",
        "reserves" => "3305",
        "results" => "3605",
        "retained" => "3610",
        _ => "3105",
    };

    // Build entity using the repository's internal types
    use crate::infrastructure::repositories::liability::Equity;
    
    let entity = Equity {
        id: id.clone(),
        equity_type: request.equity_type.clone(),
        description: request.description.clone(),
        amount: request.amount,
        account_code: Some(account_code.to_string()),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    // Save to database
    equity_state.create(&entity)?;
    eprintln!("[DEBUG] Created equity: {} - S/ {} (account: {})", entity.equity_type, entity.amount, account_code);

    // AUTOMATIC ACCOUNTING ENTRY for capital
    // When registering capital, automatically create entry to keep balance balanced
    // Debit: Asset account (Caja 1105 or Bancos 1110)
    // Credit: Capital account (3105)
    if request.equity_type == "capital" {
        // Default to Caja if not specified
        let asset_account = request.asset_account_code.unwrap_or_else(|| "1105".to_string());
        
        let entry_request = crate::application::dto::accounting::CreateEntryRequest {
            date: now.clone(),
            description: format!("Aporte de Capital - {}", request.description),
            debit_account: asset_account.clone(),  // Debe: Caja/Bancos (activo)
            credit_account: "3105".to_string(), // Haber: Capital (patrimonio)
            amount: request.amount,
            entry_type: Some(crate::domain::entities::accounting::EntryType::Automatic),
            reference: Some(format!("CAP-{}", &id[..8])),
            related_id: Some(id.clone()),
            related_type: Some("equity".to_string()),
        };
        
        // Create the accounting entry
        match accounting_state.create_entry(entry_request, "system".to_string()) {
            Ok(_) => {
                eprintln!("[DEBUG] Auto-accounting entry: Debit {} / Credit 3105 = S/ {}", asset_account, request.amount);
            }
            Err(e) => {
                eprintln!("[WARN] Failed to create auto-accounting entry for capital: {}", e);
                // Continue anyway - the equity is registered
            }
        }
    }

    Ok(EquityDto {
        id,
        equity_type: request.equity_type,
        description: request.description,
        amount: request.amount,
        account_code: Some(account_code.to_string()),
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

/// List all equities
#[tauri::command]
pub fn list_equities(
    state: State<EquityServiceState>,
) -> Result<Vec<EquityDto>, String> {
    let equities = state.list()?;

    Ok(equities
        .into_iter()
        .map(|e| EquityDto {
            id: e.id,
            equity_type: e.equity_type,
            description: e.description,
            amount: e.amount,
            account_code: e.account_code,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
        .collect())
}

// ============================================
// Fixed Assets (Activos Fijos)
// ============================================

use crate::application::dto::accounting::CreateEntryRequest;
use crate::domain::entities::accounting::EntryType;

/// Fixed asset creation request
#[derive(Debug, Deserialize)]
pub struct CreateFixedAssetCommand {
    pub name: String,
    pub asset_type: String,
    pub description: Option<String>,
    pub acquisition_date: String,
    pub acquisition_cost: f64,
    pub useful_life_years: i32,
    pub account_code: Option<String>,
    /// Account for payment: "1105" (caja), "1110" (bancos), or supplier
    pub payment_account_code: Option<String>,
}

/// Fixed asset DTO
#[derive(Debug, Serialize)]
pub struct FixedAssetDto {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub description: Option<String>,
    pub acquisition_date: String,
    pub acquisition_cost: f64,
    pub current_value: f64,
    pub useful_life_years: i32,
    pub account_code: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Create fixed asset and automatic accounting entry
#[tauri::command]
pub fn create_fixed_asset(
    accounting_state: State<AccountingServiceState>,
    request: CreateFixedAssetCommand,
) -> Result<FixedAssetDto, String> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    
    // Determine account code
    let account_code = request.account_code.unwrap_or_else(|| "1635".to_string()); // Default to Maquinaria
    let payment_account = request.payment_account_code.unwrap_or_else(|| "1105".to_string()); // Default to caja
    
    // Calculate current value (straight-line depreciation)
    let annual_depreciation = request.acquisition_cost / request.useful_life_years as f64;
    let years_used = 0; // New asset, no depreciation yet
    let current_value = request.acquisition_cost - (annual_depreciation * years_used as f64);
    
    eprintln!("[FIXED ASSET] Creating: {} - S/{} (debit: {}, credit: {})", 
        request.name, request.acquisition_cost, account_code, payment_account);
    
    // Create automatic accounting entry
    // DEBE: Activo Fijo (15xx/16xx)
    // HABER: Caja/Bancos (1105/1110)
    let entry_request = CreateEntryRequest {
        date: now.clone(),
        description: format!("Compra activo fijo: {}", request.name),
        debit_account: account_code.clone(),     // DEBE: Activo Fijo (16xx)
        credit_account: payment_account.clone(), // HABER: Caja (1105) or Bancos (1110)
        amount: request.acquisition_cost,
        entry_type: Some(EntryType::Automatic),
        reference: Some(format!("FA-{}", &id[..8])),
        related_id: Some(id.clone()),
        related_type: Some("fixed_asset".to_string()),
    };
    
    eprintln!("[FIXED ASSET] Calling create_entry...");
    
    let result = accounting_state.create_entry(entry_request, "system".to_string());
    
    match result {
        Ok(entry) => {
            eprintln!("[FIXED ASSET] SUCCESS: Debit {} (S/ {}) / Credit {} (S/ {})", 
                account_code, request.acquisition_cost, payment_account, request.acquisition_cost);
        }
        Err(e) => {
            eprintln!("[FIXED ASSET] ERROR: {}", e);
            return Err(format!("Error al crear entrada contable: {}", e));
        }
    }
    
    Ok(FixedAssetDto {
        id,
        name: request.name,
        asset_type: request.asset_type,
        description: request.description,
        acquisition_date: request.acquisition_date,
        acquisition_cost: request.acquisition_cost,
        current_value,
        useful_life_years: request.useful_life_years,
        account_code,
        status: "active".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}
