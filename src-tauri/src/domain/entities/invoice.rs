//! Invoice Entity - Domain Model
//!
//! Pure domain entities for invoicing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Invoice status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Pending,   // Pendiente de pago
    Paid,      // Pagada
    Overdue,   // Vencida
    Cancelled, // Anulada
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvoiceStatus::Pending => "pending",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Overdue => "overdue",
            InvoiceStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(InvoiceStatus::Pending),
            "paid" => Some(InvoiceStatus::Paid),
            "overdue" => Some(InvoiceStatus::Overdue),
            "cancelled" => Some(InvoiceStatus::Cancelled),
            _ => None,
        }
    }
}

/// Payment method for invoices (renamed to avoid conflict with payment module)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvoicePaymentMethod {
    Cash,     // Efectivo
    Transfer, // Transferencia bancaria
    Card,     // Tarjeta
}

impl InvoicePaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvoicePaymentMethod::Cash => "cash",
            InvoicePaymentMethod::Transfer => "transfer",
            InvoicePaymentMethod::Card => "card",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cash" => Some(InvoicePaymentMethod::Cash),
            "transfer" => Some(InvoicePaymentMethod::Transfer),
            "card" => Some(InvoicePaymentMethod::Card),
            _ => None,
        }
    }
}

/// Invoice - main invoice entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub series: String,      // Invoice series (e.g., "F001")
    pub number: String,      // Invoice number (e.g., "00001")
    pub client_name: String, // Client business name
    pub client_ruc: String,  // Client RUC number
    pub client_address: Option<String>,
    pub emission_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub subtotal: f64, // Subtotal before IGV
    pub igv: f64,      // IGV (18%)
    pub total: f64,    // Total including IGV
    pub status: InvoiceStatus,
    pub payment_method: Option<InvoicePaymentMethod>,
    pub paid_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl Invoice {
    /// Create a new invoice
    pub fn new(
        id: String,
        series: String,
        number: String,
        client_name: String,
        client_ruc: String,
        emission_date: DateTime<Utc>,
        due_date: DateTime<Utc>,
        created_by: String,
    ) -> Self {
        Self {
            id,
            series,
            number,
            client_name,
            client_ruc,
            client_address: None,
            emission_date,
            due_date,
            subtotal: 0.0,
            igv: 0.0,
            total: 0.0,
            status: InvoiceStatus::Pending,
            payment_method: None,
            paid_date: None,
            created_at: Utc::now(),
            created_by,
        }
    }

    /// Set client address
    pub fn set_client_address(&mut self, address: String) {
        self.client_address = Some(address);
    }

    /// Calculate totals from subtotal (IGV is 18%)
    pub fn calculate_totals(&mut self, subtotal: f64) {
        self.subtotal = subtotal;
        self.igv = subtotal * constants::IGV_RATE;
        self.total = subtotal + self.igv;
    }

    /// Register payment
    pub fn register_payment(
        &mut self,
        payment_method: InvoicePaymentMethod,
        paid_date: DateTime<Utc>,
    ) {
        self.payment_method = Some(payment_method);
        self.paid_date = Some(paid_date);
        self.status = InvoiceStatus::Paid;
    }

    /// Mark as overdue
    pub fn mark_overdue(&mut self) {
        if self.status == InvoiceStatus::Pending {
            self.status = InvoiceStatus::Overdue;
        }
    }

    /// Cancel invoice
    pub fn cancel(&mut self) {
        self.status = InvoiceStatus::Cancelled;
    }

    /// Check if invoice is paid
    pub fn is_paid(&self) -> bool {
        self.status == InvoiceStatus::Paid
    }

    /// Check if invoice is overdue
    pub fn is_overdue(&self) -> bool {
        self.status == InvoiceStatus::Pending && Utc::now() > self.due_date
    }

    /// Get full invoice number (series + number)
    pub fn full_number(&self) -> String {
        format!("{}-{}", self.series, self.number)
    }

    /// Get formatted total
    pub fn formatted_total(&self) -> String {
        format!("S/ {:.2}", self.total)
    }
}

/// InvoiceLine - individual line item in an invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    pub id: String,
    pub invoice_id: String,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
    pub created_at: DateTime<Utc>,
}

impl InvoiceLine {
    /// Create a new invoice line
    pub fn new(
        id: String,
        invoice_id: String,
        description: String,
        quantity: f64,
        unit_price: f64,
    ) -> Self {
        Self {
            id,
            invoice_id,
            description,
            quantity,
            unit_price,
            total: quantity * unit_price,
            created_at: Utc::now(),
        }
    }

    /// Update quantity
    pub fn set_quantity(&mut self, quantity: f64) {
        self.quantity = quantity;
        self.total = quantity * self.unit_price;
    }

    /// Update unit price
    pub fn set_unit_price(&mut self, unit_price: f64) {
        self.unit_price = unit_price;
        self.total = self.quantity * unit_price;
    }

    /// Update description
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }
}

/// Invoice constants
pub mod constants {
    /// IGV rate (Impuesto General a las Ventas)
    pub const IGV_RATE: f64 = 0.18; // 18%

    /// Default payment term in days
    pub const DEFAULT_PAYMENT_TERM_DAYS: i64 = 30;
}

/// Calculate IGV amount
pub fn calculate_igv(subtotal: f64) -> f64 {
    subtotal * constants::IGV_RATE
}

/// Calculate total including IGV
pub fn calculate_total(subtotal: f64) -> f64 {
    subtotal * (1.0 + constants::IGV_RATE)
}

/// Generate invoice number (padding with zeros)
pub fn generate_invoice_number(number: u32, padding: u32) -> String {
    format!("{:0>width$}", number, width = padding as usize)
}

/// Validate RUC format (11 digits)
pub fn validate_ruc(ruc: &str) -> bool {
    ruc.len() == 11 && ruc.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_creation() {
        let invoice = Invoice::new(
            "inv-id".to_string(),
            "F001".to_string(),
            "00001".to_string(),
            "Empresa ABC".to_string(),
            "20123456789".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(30),
            "user-id".to_string(),
        );

        assert_eq!(invoice.series, "F001");
        assert_eq!(invoice.full_number(), "F001-00001");
    }

    #[test]
    fn test_calculate_totals() {
        let mut invoice = Invoice::new(
            "inv-id".to_string(),
            "F001".to_string(),
            "00001".to_string(),
            "Empresa ABC".to_string(),
            "20123456789".to_string(),
            Utc::now(),
            Utc::now(),
            "user-id".to_string(),
        );

        invoice.calculate_totals(1000.0);

        assert_eq!(invoice.subtotal, 1000.0);
        assert_eq!(invoice.igv, 180.0);
        assert_eq!(invoice.total, 1180.0);
    }

    #[test]
    fn test_register_payment() {
        let mut invoice = Invoice::new(
            "inv-id".to_string(),
            "F001".to_string(),
            "00001".to_string(),
            "Empresa ABC".to_string(),
            "20123456789".to_string(),
            Utc::now(),
            Utc::now(),
            "user-id".to_string(),
        );

        invoice.register_payment(InvoicePaymentMethod::Transfer, Utc::now());

        assert!(invoice.is_paid());
        assert!(invoice.paid_date.is_some());
    }

    #[test]
    fn test_invoice_line() {
        let line = InvoiceLine::new(
            "line-id".to_string(),
            "inv-id".to_string(),
            "Mensualidad Abril".to_string(),
            1.0,
            500.0,
        );

        assert_eq!(line.total, 500.0);
    }

    #[test]
    fn test_calculate_igv() {
        let igv = calculate_igv(1000.0);
        assert!((igv - 180.0).abs() < 0.01);
    }

    #[test]
    fn test_validate_ruc() {
        assert!(validate_ruc("20123456789"));
        assert!(!validate_ruc("2012345678")); // Too short
        assert!(!validate_ruc("2012345678a")); // Contains letter
    }

    #[test]
    fn test_generate_invoice_number() {
        assert_eq!(generate_invoice_number(1, 5), "00001");
        assert_eq!(generate_invoice_number(42, 5), "00042");
        assert_eq!(generate_invoice_number(1000, 5), "01000");
    }
}
