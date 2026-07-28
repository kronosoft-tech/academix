//! Payment Use Cases

use crate::application::dto::{
    CreatePaymentRequest, PaymentDelinquencyStatus, PaymentDto, PaymentStatusDto,
    UpdatePaymentRequest,
};
use crate::application::errors::ApplicationError;
use crate::application::ports::{CourseRepository, GroupRepository, PaymentRepository};
use crate::domain::entities::{Payment, PaymentMethod};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Payment service
pub struct PaymentService<R: PaymentRepository, G: GroupRepository, C: CourseRepository> {
    payment_repository: R,
    group_repository: G,
    course_repository: C,
}

impl<R: PaymentRepository, G: GroupRepository, C: CourseRepository> PaymentService<R, G, C> {
    pub fn new(payment_repository: R, group_repository: G, course_repository: C) -> Self {
        Self {
            payment_repository,
            group_repository,
            course_repository,
        }
    }

    /// Create a new payment
    pub async fn create(&self, request: CreatePaymentRequest) -> Result<PaymentDto, ApplicationError> {
        // Default to "cash" if method is not provided
        let method_str = request.method.unwrap_or_else(|| "cash".to_string());
        let method = PaymentMethod::from_str(&method_str)
            .ok_or_else(|| ApplicationError::Validation("Invalid payment method".to_string()))?;

        // Get the group to find start_date
        let group = self
            .group_repository
            .find_by_id(&request.group_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Group not found".to_string()))?;

        // Get existing payments count for this student in this group to calculate sequence
        let existing_payments = self
            .payment_repository
            .find_by_student_id(&request.student_id)
            .await?
            .into_iter()
            .filter(|p| p.group_id == request.group_id)
            .collect::<Vec<_>>();

        let payment_sequence = existing_payments.len() + 1; // 1-based: first payment is month 1

        // Calculate due_date: start_date + (payment_sequence - 1) months
        let due_date = if !request.due_date.is_empty() {
            // If user provided due_date, use it
            request.due_date.clone()
        } else {
            // Calculate from group start_date + sequence
            let start_date = group
                .start_date
                .clone()
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

            // Parse the start_date and add (sequence - 1) months
            if let Ok(dt) = DateTime::parse_from_rfc3339(&start_date) {
                let months_to_add = (payment_sequence - 1) as i32;
                let new_date = dt
                    .checked_add_months(chrono::Months::new(months_to_add as u32))
                    .unwrap_or(dt);
                new_date.to_rfc3339()
            } else {
                // Fallback to current date + sequence months
                let now = chrono::Utc::now();
                let new_date = now + chrono::Duration::days((payment_sequence as i64 - 1) * 30);
                new_date.to_rfc3339()
            }
        };

        let mut payment = Payment::new(
            Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string(),
            request.student_id,
            request.group_id,
            request.amount,
            method,
        );

        // Set the calculated due_date
        payment.set_due_date(due_date.clone());

        // If paid=true, mark as paid immediately with auto-generated reference
        if request.paid.unwrap_or(false) {
            payment.mark_paid();
            // Generate reference: PAG-YYYYMMDD-XXXXX (5 random chars from UUID)
            let now = chrono::Utc::now();
            let uuid_part = Uuid::new_v4()
                .to_string()
                .replace("-", "")
                .chars()
                .take(5)
                .collect::<String>();
            payment.reference = Some(format!("PAG-{}-{}", now.format("%Y%m%d"), uuid_part));
            payment.description = request.description;
        }

        self.payment_repository.save(&payment).await?;

        Ok(PaymentDto {
            id: payment.id,
            student_id: payment.student_id,
            group_id: payment.group_id,
            amount: payment.amount,
            method: payment.method.as_str().to_string(),
            status: payment.status.as_str().to_string(),
            due_date,
            paid_at: payment.paid_at.map(|dt| dt.to_rfc3339()),
            description: payment.description,
        })
    }

    /// Get payment by ID
    pub async fn get_by_id(&self, id: &str) -> Result<PaymentDto, ApplicationError> {
        let payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Payment not found".to_string()))?;

        Ok(PaymentDto {
            id: payment.id,
            student_id: payment.student_id,
            group_id: payment.group_id,
            amount: payment.amount,
            method: payment.method.as_str().to_string(),
            status: payment.status.as_str().to_string(),
            due_date: payment.due_date.unwrap_or_default(),
            paid_at: payment.paid_at.map(|dt| dt.to_rfc3339()),
            description: payment.description,
        })
    }

    /// List all payments
    pub async fn list(&self) -> Result<Vec<PaymentDto>, ApplicationError> {
        let payments = self.payment_repository.find_all().await?;

        Ok(payments
            .into_iter()
            .map(|p| PaymentDto {
                id: p.id,
                student_id: p.student_id,
                group_id: p.group_id,
                amount: p.amount,
                method: p.method.as_str().to_string(),
                status: p.status.as_str().to_string(),
                due_date: p.due_date.unwrap_or_default(),
                paid_at: p.paid_at.map(|dt| dt.to_rfc3339()),
                description: p.description,
            })
            .collect())
    }

    /// List payments by student
    pub async fn list_by_student(&self, student_id: &str) -> Result<Vec<PaymentDto>, ApplicationError> {
        let payments = self.payment_repository.find_by_student_id(student_id).await?;

        Ok(payments
            .into_iter()
            .map(|p| PaymentDto {
                id: p.id,
                student_id: p.student_id,
                group_id: p.group_id,
                amount: p.amount,
                method: p.method.as_str().to_string(),
                status: p.status.as_str().to_string(),
                due_date: p.due_date.unwrap_or_default(),
                paid_at: p.paid_at.map(|dt| dt.to_rfc3339()),
                description: p.description,
            })
            .collect())
    }

    /// Update payment (e.g., mark as paid)
    pub async fn update(
        &self,
        id: &str,
        request: UpdatePaymentRequest,
    ) -> Result<PaymentDto, ApplicationError> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Payment not found".to_string()))?;

        if let Some(status) = request.status {
            payment.status =
                crate::domain::entities::PaymentStatus::from_str(&status).unwrap_or(payment.status);
            if status == "paid" {
                payment.paid_at = Some(Utc::now());
            }
        }

        // Handle reference if provided
        if let Some(ref r) = request.reference {
            payment.reference = Some(r.clone());
        }

        // Handle paid_date if provided
        if let Some(ref pd) = request.paid_date {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(pd) {
                payment.paid_at = Some(dt.with_timezone(&Utc));
            }
        }

        self.payment_repository.update(&payment).await?;

        Ok(PaymentDto {
            id: payment.id,
            student_id: payment.student_id,
            group_id: payment.group_id,
            amount: payment.amount,
            method: payment.method.as_str().to_string(),
            status: payment.status.as_str().to_string(),
            due_date: payment.due_date.unwrap_or_default(),
            paid_at: payment.paid_at.map(|dt| dt.to_rfc3339()),
            description: payment.description,
        })
    }

    /// Delete payment
    pub async fn delete(&self, id: &str) -> Result<(), ApplicationError> {
        self.payment_repository.delete(id).await?;
        Ok(())
    }

    /// List all payments as domain entities (with all fields including reference)
    pub async fn list_domain(&self) -> Result<Vec<Payment>, ApplicationError> {
        Ok(self.payment_repository.find_all().await?)
    }

    /// Calculate payment status based on due_date and payment history
    pub async fn calculate_payment_status(
        &self,
        student_id: &str,
        group_id: &str,
    ) -> Result<PaymentStatusDto, ApplicationError> {
        // Get the group to find start_date
        let group = self
            .group_repository
            .find_by_id(group_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound("Group not found".to_string()))?;

        // Get the course to find price
        let course_price = self
            .course_repository
            .find_by_id(&group.course_id)
            .await?
            .map(|c| c.price)
            .unwrap_or(0.0);

        // Get student payments
        let payments = self.payment_repository.find_by_student_id(student_id).await?;

        // Calculate total paid (only completed payments)
        let total_paid: f64 = payments
            .iter()
            .filter(|p| p.status.as_str() == "paid")
            .map(|p| p.amount)
            .sum();

        // Calculate months paid (one payment = one month)
        let months_paid = payments
            .iter()
            .filter(|p| p.status.as_str() == "paid")
            .count() as i32;

        // Calculate debt
        let debt_amount = if course_price > 0.0 {
            // If course price is monthly, multiply by months_paid
            let total_expected = course_price * months_paid as f64;
            (total_expected - total_paid).max(0.0)
        } else {
            0.0
        };

        let student_name = "Student".to_string();
        let group_name = group.name.clone();
        let start_date_str = match group.start_date {
            Some(s) => s.clone(),
            None => "".to_string(),
        };

        // Calculate next payment date based on months paid
        let next_payment_date = if !start_date_str.is_empty() {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&start_date_str) {
                let next_date = dt.checked_add_months(chrono::Months::new(months_paid as u32));
                next_date.map(|d| d.to_rfc3339())
            } else {
                None
            }
        } else {
            None
        };

        // Calculate days delayed
        let now = Utc::now();
        let due_date = if start_date_str.is_empty() {
            now
        } else {
            DateTime::parse_from_rfc3339(&start_date_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now)
        };

        let days_delayed = (now - due_date).num_days() as i32;

        // Determine status - now based on debt
        let status = if debt_amount > 0.0 && debt_amount >= course_price {
            // If owes at least one full monthly payment
            PaymentDelinquencyStatus::Delinquent
        } else if debt_amount > 0.0 {
            // Partial payment made
            PaymentDelinquencyStatus::Current
        } else if months_paid > 0
            && (course_price == 0.0 || total_paid >= course_price * months_paid as f64)
        {
            // Paid ahead
            PaymentDelinquencyStatus::Ahead
        } else {
            PaymentDelinquencyStatus::Current
        };

        Ok(PaymentStatusDto {
            student_id: student_id.to_string(),
            student_name,
            group_name,
            group_id: group_id.to_string(),
            due_date: start_date_str.clone(),
            next_payment_date,
            status,
            days_delayed,
            total_paid,
            course_price,
            debt_amount,
            months_paid,
        })
    }

    /// Get all students with payment summary
    pub async fn get_all_students_payment_summary(
        &self,
    ) -> Result<Vec<PaymentStatusDto>, ApplicationError> {
        let mut summaries = Vec::new();

        let groups = self.group_repository.find_all().await?;

        for group in groups {
            // Get course price
            let course_price = self
                .course_repository
                .find_by_id(&group.course_id)
                .await?
                .map(|c| c.price)
                .unwrap_or(0.0);

            let payments = self.payment_repository.find_by_group_id(&group.id).await?;

            let mut student_payments: std::collections::HashMap<String, Vec<Payment>> =
                std::collections::HashMap::new();
            for payment in payments {
                student_payments
                    .entry(payment.student_id.clone())
                    .or_default()
                    .push(payment);
            }

            for (student_id, student_payments) in student_payments {
                // Calculate total paid (only completed payments)
                let total_paid: f64 = student_payments
                    .iter()
                    .filter(|p| p.status.as_str() == "paid")
                    .map(|p| p.amount)
                    .sum();

                // Calculate months paid
                let months_paid = student_payments
                    .iter()
                    .filter(|p| p.status.as_str() == "paid")
                    .count() as i32;

                // Calculate debt
                let debt_amount = if course_price > 0.0 {
                    let total_expected = course_price * months_paid as f64;
                    (total_expected - total_paid).max(0.0)
                } else {
                    0.0
                };

                let start_date_str = group.start_date.as_deref().unwrap_or("").to_string();

                // Calculate next payment date
                let next_payment_date = if !start_date_str.is_empty() {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(&start_date_str) {
                        let next_date =
                            dt.checked_add_months(chrono::Months::new(months_paid as u32));
                        next_date.map(|d| d.to_rfc3339())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let now = Utc::now();
                let due_date = if start_date_str.is_empty() {
                    now
                } else {
                    DateTime::parse_from_rfc3339(&start_date_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(now)
                };

                let days_delayed = (now - due_date).num_days() as i32;

                // Determine status based on debt
                let status = if debt_amount > 0.0 && debt_amount >= course_price {
                    PaymentDelinquencyStatus::Delinquent
                } else if debt_amount > 0.0 {
                    PaymentDelinquencyStatus::Current
                } else if months_paid > 0
                    && (course_price == 0.0 || total_paid >= course_price * months_paid as f64)
                {
                    PaymentDelinquencyStatus::Ahead
                } else {
                    PaymentDelinquencyStatus::Current
                };

                summaries.push(PaymentStatusDto {
                    student_id,
                    student_name: "Student".to_string(),
                    group_name: group.name.clone(),
                    group_id: group.id.clone(),
                    due_date: start_date_str.clone(),
                    next_payment_date,
                    status,
                    days_delayed,
                    total_paid,
                    course_price,
                    debt_amount,
                    months_paid,
                });
            }
        }

        Ok(summaries)
    }
}
