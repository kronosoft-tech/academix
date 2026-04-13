//! Invoice Repository Ports (Trait Definitions)
//!
//! In-memory repository implementations for invoices.

use crate::application::ports::invoice::{InvoiceLineRepository, InvoiceRepository};
use crate::domain::entities::invoice::{Invoice, InvoiceLine, InvoicePaymentMethod, InvoiceStatus};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory implementation of InvoiceRepository
pub struct InMemoryInvoiceRepository {
    invoices: Arc<RwLock<HashMap<String, Invoice>>>,
    next_id: Arc<RwLock<u32>>,
    next_number: Arc<RwLock<HashMap<String, u32>>>,
}

impl InMemoryInvoiceRepository {
    pub fn new() -> Self {
        Self {
            invoices: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            next_number: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn generate_id(&self) -> String {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("inv-{:03}", *counter);
        *counter += 1;
        id
    }

    pub fn seed_demo_invoices(&self) {
        let now = chrono::Utc::now();

        let demo_invoices = vec![
            Invoice::new(
                "inv-001".to_string(),
                "F001".to_string(),
                "00001".to_string(),
                "Colegio San Marcos".to_string(),
                "20123456789".to_string(),
                now - chrono::Duration::days(5),
                now + chrono::Duration::days(25),
                "admin".to_string(),
            ),
            Invoice::new(
                "inv-002".to_string(),
                "F001".to_string(),
                "00002".to_string(),
                "Instituto Superior".to_string(),
                "20187654321".to_string(),
                now - chrono::Duration::days(15),
                now + chrono::Duration::days(15),
                "admin".to_string(),
            ),
        ];

        let mut invoices = self.invoices.write().unwrap();
        let mut numbers = self.next_number.write().unwrap();
        numbers.insert("F001".to_string(), 3);

        for mut inv in demo_invoices {
            if inv.id.is_empty() {
                inv.id = self.generate_id();
            }
            if inv.series == "F001" && inv.number == "00001" {
                inv.subtotal = 5000.0;
                inv.igv = 900.0;
                inv.total = 5900.0;
            } else if inv.series == "F001" && inv.number == "00002" {
                inv.subtotal = 3000.0;
                inv.igv = 540.0;
                inv.total = 3540.0;
                inv.status = InvoiceStatus::Paid;
                inv.payment_method = Some(InvoicePaymentMethod::Transfer);
                inv.paid_date = Some(now - chrono::Duration::days(10));
            }
            invoices.insert(inv.id.clone(), inv);
        }
    }
}

impl InvoiceRepository for InMemoryInvoiceRepository {
    fn create(&self, mut invoice: Invoice) -> Result<Invoice, String> {
        if invoice.id.is_empty() {
            invoice.id = self.generate_id();
        }
        if invoice.number.is_empty() {
            let mut numbers = self.next_number.write().unwrap();
            let num = numbers.entry(invoice.series.clone()).or_insert(1);
            invoice.number = format!("{:05}", *num);
            *num += 1;
        }
        let mut invoices = self.invoices.write().unwrap();
        invoices.insert(invoice.id.clone(), invoice.clone());
        Ok(invoice)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Invoice>, String> {
        let invoices = self.invoices.read().unwrap();
        Ok(invoices.get(id).cloned())
    }

    fn get_by_series_number(&self, series: &str, number: &str) -> Result<Option<Invoice>, String> {
        let invoices = self.invoices.read().unwrap();
        Ok(invoices
            .values()
            .find(|i| i.series == series && i.number == number)
            .cloned())
    }

    fn list(
        &self,
        status: Option<InvoiceStatus>,
        client_ruc: Option<&str>,
        _date_from: Option<&str>,
        _date_to: Option<&str>,
    ) -> Result<Vec<Invoice>, String> {
        let invoices = self.invoices.read().unwrap();
        let mut result: Vec<Invoice> = invoices
            .values()
            .filter(|i| {
                let status_match = status.map_or(true, |s| i.status == s);
                let ruc_match = client_ruc.map_or(true, |r| i.client_ruc == r);
                status_match && ruc_match
            })
            .cloned()
            .collect();
        result.sort_by(|a, b| b.emission_date.cmp(&a.emission_date));
        Ok(result)
    }

    fn list_by_client(&self, client_ruc: &str) -> Result<Vec<Invoice>, String> {
        let invoices = self.invoices.read().unwrap();
        let mut result: Vec<Invoice> = invoices
            .values()
            .filter(|i| i.client_ruc == client_ruc)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.emission_date.cmp(&a.emission_date));
        Ok(result)
    }

    fn update(&self, invoice: Invoice) -> Result<Invoice, String> {
        let mut invoices = self.invoices.write().unwrap();
        if invoices.contains_key(&invoice.id) {
            invoices.insert(invoice.id.clone(), invoice.clone());
            Ok(invoice)
        } else {
            Err(format!("Invoice not found: {}", invoice.id))
        }
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut invoices = self.invoices.write().unwrap();
        if let Some(invoice) = invoices.get_mut(id) {
            invoice.cancel();
            Ok(true)
        } else {
            Err(format!("Invoice not found: {}", id))
        }
    }

    fn get_next_number(&self, series: &str) -> Result<String, String> {
        let mut numbers = self.next_number.write().unwrap();
        let num = numbers.entry(series.to_string()).or_insert(1);
        let result = format!("{:05}", *num);
        *num += 1;
        Ok(result)
    }

    fn get_total_pending(&self) -> Result<f64, String> {
        let invoices = self.invoices.read().unwrap();
        let total: f64 = invoices
            .values()
            .filter(|i| i.status == InvoiceStatus::Pending)
            .map(|i| i.total)
            .sum();
        Ok(total)
    }

    fn get_total_by_status(&self, status: InvoiceStatus) -> Result<f64, String> {
        let invoices = self.invoices.read().unwrap();
        let total: f64 = invoices
            .values()
            .filter(|i| i.status == status)
            .map(|i| i.total)
            .sum();
        Ok(total)
    }
}

/// In-memory implementation of InvoiceLineRepository
pub struct InMemoryInvoiceLineRepository {
    lines: Arc<RwLock<HashMap<String, InvoiceLine>>>,
    next_id: Arc<RwLock<u32>>,
}

impl InMemoryInvoiceLineRepository {
    pub fn new() -> Self {
        Self {
            lines: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    fn generate_id(&self) -> String {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("invline-{:03}", *counter);
        *counter += 1;
        id
    }
}

impl InvoiceLineRepository for InMemoryInvoiceLineRepository {
    fn create(&self, mut line: InvoiceLine) -> Result<InvoiceLine, String> {
        if line.id.is_empty() {
            line.id = self.generate_id();
        }
        let mut lines = self.lines.write().unwrap();
        lines.insert(line.id.clone(), line.clone());
        Ok(line)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<InvoiceLine>, String> {
        let lines = self.lines.read().unwrap();
        Ok(lines.get(id).cloned())
    }

    fn get_by_invoice(&self, invoice_id: &str) -> Result<Vec<InvoiceLine>, String> {
        let lines = self.lines.read().unwrap();
        let result: Vec<InvoiceLine> = lines
            .values()
            .filter(|l| l.invoice_id == invoice_id)
            .cloned()
            .collect();
        Ok(result)
    }

    fn update(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let mut lines = self.lines.write().unwrap();
        if lines.contains_key(&line.id) {
            lines.insert(line.id.clone(), line.clone());
            Ok(line)
        } else {
            Err(format!("Invoice line not found: {}", line.id))
        }
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut lines = self.lines.write().unwrap();
        if lines.remove(id).is_some() {
            Ok(true)
        } else {
            Err(format!("Invoice line not found: {}", id))
        }
    }

    fn delete_by_invoice(&self, invoice_id: &str) -> Result<bool, String> {
        let mut lines = self.lines.write().unwrap();
        let count_before = lines.len();
        lines.retain(|_, l| l.invoice_id != invoice_id);
        Ok(lines.len() < count_before)
    }
}
