//! Domain Entities - Academix MVP
//!
//! Pure domain models with no framework annotations or persistence concerns.

pub mod accounting;
pub mod attendance;
pub mod course;
pub mod employee;
pub mod group;
pub mod invoice;
pub mod payment;
pub mod payroll;
pub mod student;
pub mod user;

// Export specific items to avoid conflicts (e.g., PaymentMethod exists in both payment and invoice)
pub use accounting::*;
pub use attendance::*;
pub use course::*;
pub use employee::*;
pub use group::*;
pub use invoice::{
    calculate_igv, calculate_total, generate_invoice_number, validate_ruc, Invoice, InvoiceLine,
    InvoicePaymentMethod, InvoiceStatus,
};
pub use payment::*;
pub use payroll::*;
pub use student::*;
pub use user::*;
