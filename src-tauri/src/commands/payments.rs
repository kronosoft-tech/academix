use serde::{Deserialize, Serialize};
use tauri::State;

use chrono::Utc;
use crate::application::dto::accounting::{CreateEntryRequest, AccountingEntryDto};
use crate::application::dto::{
    CreatePaymentRequest, PaymentDto, PaymentStatusDto, UpdatePaymentRequest,
};
use crate::application::use_cases::AccountingService;
use crate::application::use_cases::PaymentService;
use crate::infrastructure::repositories::{
    MemoryBackedAccountingEntryRepository, MemoryBackedCourseRepository,
    MemoryBackedGroupRepository, MemoryBackedPaymentRepository,
};

pub type PaymentServiceState =
    PaymentService<MemoryBackedPaymentRepository, MemoryBackedGroupRepository, MemoryBackedCourseRepository>;

#[derive(Debug, Deserialize)]
pub struct CreatePaymentCommand {
    pub student_id: String,
    pub group_id: String,
    pub amount: f64,
    pub method: Option<String>,
    pub due_date: Option<String>,
    pub description: Option<String>,
    pub paid: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdatePaymentCommand {
    pub status: Option<String>,
    pub reference: Option<String>,
    pub paid_date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentCommandResponse {
    pub success: bool,
    pub data: Option<PaymentDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<PaymentDto>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentStatusCommandResponse {
    pub success: bool,
    pub data: Option<PaymentStatusDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentStatusListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<PaymentStatusDto>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncPaymentsAccountingResponse {
    pub success: bool,
    pub synced: usize,
    pub skipped: usize,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn create_payment(
    state: State<'_, PaymentServiceState>,
    request: CreatePaymentCommand,
) -> Result<PaymentCommandResponse, String> {
    match state.create(CreatePaymentRequest {
        student_id: request.student_id,
        group_id: request.group_id,
        amount: request.amount,
        method: request.method,
        due_date: request.due_date.unwrap_or_default(),
        description: request.description,
        paid: request.paid,
    }).await {
        Ok(payment) => Ok(PaymentCommandResponse {
            success: true,
            data: Some(payment),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_payment(state: State<'_, PaymentServiceState>, id: String) -> Result<PaymentCommandResponse, String> {
    match state.get_by_id(&id).await {
        Ok(payment) => Ok(PaymentCommandResponse {
            success: true,
            data: Some(payment),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_payments(state: State<'_, PaymentServiceState>) -> Result<PaymentListCommandResponse, String> {
    match state.list().await {
        Ok(payments) => Ok(PaymentListCommandResponse {
            success: true,
            data: Some(payments),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn list_payments_by_student(
    state: State<'_, PaymentServiceState>,
    student_id: String,
) -> Result<PaymentListCommandResponse, String> {
    match state.list_by_student(&student_id).await {
        Ok(payments) => Ok(PaymentListCommandResponse {
            success: true,
            data: Some(payments),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_payment(
    state: State<'_, PaymentServiceState>,
    id: String,
    request: UpdatePaymentRequest,
    accounting_entry_state: State<'_, MemoryBackedAccountingEntryRepository>,
) -> Result<PaymentCommandResponse, String> {
    let update_result = state.update(
        &id,
        UpdatePaymentRequest {
            status: request.status.clone(),
            reference: request.reference.clone(),
            paid_date: request.paid_date.clone(),
        },
    ).await;

    match update_result {
        Ok(payment) => {
            if request.status.as_deref() == Some("paid") {
                let accounting_service = AccountingService::new(
                    accounting_entry_state.inner().clone(),
                );

                let existing_entries = list_accounting_entries_by_reference(
                    accounting_entry_state.inner(),
                    &payment.id,
                ).await;

                if existing_entries.is_empty() {
                    let entry_request = CreateEntryRequest {
                        date: Utc::now().format("%Y-%m-%d").to_string(),
                        entry_type: "income".to_string(),
                        category: "tuition".to_string(),
                        description: format!("Pago estudiante - {}", id),
                        amount: payment.amount,
                        reference: Some(format!("PAG-{}", &payment.id[..8.min(payment.id.len())])),
                    };

                    match accounting_service.create_entry(entry_request).await {
                        Ok(_entry) => {}
                        Err(e) => {
                            return Ok(PaymentCommandResponse {
                                success: true,
                                data: Some(payment),
                                error: Some(format!("Pago actualizado pero error contable: {}", e)),
                            });
                        }
                    }
                }
            }

            Ok(PaymentCommandResponse {
                success: true,
                data: Some(payment),
                error: None,
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_payment(
    state: State<'_, PaymentServiceState>,
    id: String,
    accounting_entry_state: State<'_, MemoryBackedAccountingEntryRepository>,
) -> Result<PaymentCommandResponse, String> {
    if let Err(e) = delete_accounting_entries_by_reference(accounting_entry_state.inner(), &id).await {
        eprintln!("[DEBUG] Failed to delete related accounting entries: {}", e);
    }

    match state.delete(&id).await {
        Ok(()) => Ok(PaymentCommandResponse {
            success: true,
            data: None,
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn register_payment_with_income(
    state: State<'_, PaymentServiceState>,
    payment_id: String,
    reference: String,
    accounting_entry_state: State<'_, MemoryBackedAccountingEntryRepository>,
) -> Result<PaymentCommandResponse, String> {
    let update_result = state.update(
        &payment_id,
        UpdatePaymentRequest {
            status: Some("paid".to_string()),
            reference: Some(reference.clone()),
            paid_date: Some(Utc::now().format("%Y-%m-%d").to_string()),
        },
    ).await;

    match update_result {
        Ok(_payment) => {
            let accounting_service = AccountingService::new(
                accounting_entry_state.inner().clone(),
            );

            let entry_request = CreateEntryRequest {
                date: Utc::now().format("%Y-%m-%d").to_string(),
                entry_type: "income".to_string(),
                category: "tuition".to_string(),
                description: format!("Pago {}", _payment.student_id),
                amount: _payment.amount,
                reference: Some(format!("PAG-{}", reference)),
            };

            match accounting_service.create_entry(entry_request).await {
                Ok(_entry) => Ok(PaymentCommandResponse {
                    success: true,
                    data: Some(_payment),
                    error: None,
                }),
                Err(e) => {
                    Ok(PaymentCommandResponse {
                        success: true,
                        data: Some(_payment),
                        error: Some(format!(
                            "Warning: Payment updated but accounting entry failed: {}",
                            e
                        )),
                    })
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_student_payment_status(
    state: State<'_, PaymentServiceState>,
    student_id: String,
    group_id: String,
) -> Result<PaymentStatusCommandResponse, String> {
    match state.calculate_payment_status(&student_id, &group_id).await {
        Ok(status) => Ok(PaymentStatusCommandResponse {
            success: true,
            data: Some(status),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_all_students_payment_summary(
    state: State<'_, PaymentServiceState>,
) -> Result<PaymentStatusListCommandResponse, String> {
    match state.get_all_students_payment_summary().await {
        Ok(summaries) => Ok(PaymentStatusListCommandResponse {
            success: true,
            data: Some(summaries),
            error: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn sync_payments_to_accounting(
    state: State<'_, PaymentServiceState>,
    accounting_entry_state: State<'_, MemoryBackedAccountingEntryRepository>,
) -> Result<SyncPaymentsAccountingResponse, String> {
    let payments = match state.list_domain().await {
        Ok(p) => p,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let accounting_service = AccountingService::new(
        accounting_entry_state.inner().clone(),
    );

    let mut synced = 0;
    let mut skipped = 0;

    for payment in payments {
        if payment.status.as_str() != "paid" {
            skipped += 1;
            continue;
        }

        let existing = list_accounting_entries_by_reference(
            accounting_entry_state.inner(),
            &payment.id,
        ).await;

        if !existing.is_empty() {
            skipped += 1;
            continue;
        }

        let date_str = payment
            .paid_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

        let reference = payment.reference.clone()
            .or_else(|| Some(format!("PAG-{}", &payment.id[..8.min(payment.id.len())])));

        let entry_request = CreateEntryRequest {
            date: date_str,
            entry_type: "income".to_string(),
            category: "tuition".to_string(),
            description: format!("Pago estudiante - {}", payment.student_id),
            amount: payment.amount,
            reference,
        };

        match accounting_service.create_entry(entry_request).await {
            Ok(_) => {
                synced += 1;
            }
            Err(e) => {
                eprintln!(
                    "[DEBUG] Failed to create entry for payment {}: {}",
                    payment.id, e
                );
            }
        }
    }

    Ok(SyncPaymentsAccountingResponse {
        success: true,
        synced,
        skipped,
        error: None,
    })
}

async fn list_accounting_entries_by_reference(
    repo: &MemoryBackedAccountingEntryRepository,
    payment_id: &str,
) -> Vec<AccountingEntryDto> {
    use crate::application::ports::accounting::AccountingEntryRepository;

    match repo.list(None, None, None).await {
        Ok(entries) => entries
            .into_iter()
            .filter(|e| {
                e.reference.as_ref().map(|r| r.starts_with("PAG-")).unwrap_or(false)
                    && e.description.contains(&payment_id[..8.min(payment_id.len())])
            })
            .map(|e| AccountingEntryDto::from(e))
            .collect(),
        Err(_) => vec![],
    }
}

async fn delete_accounting_entries_by_reference(
    repo: &MemoryBackedAccountingEntryRepository,
    payment_id: &str,
) -> Result<(), String> {
    use crate::application::ports::accounting::AccountingEntryRepository;

    let entries = list_accounting_entries_by_reference(repo, payment_id).await;
    for entry in entries {
        repo.delete(&entry.id).await?;
    }
    Ok(())
}
