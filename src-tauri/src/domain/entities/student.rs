//! Student Entity - Domain Model
//!
//! Pure domain entity with no persistence concerns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Student entity - represents a student in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: String,
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub document_type: String,
    pub document_number: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub birth_date: Option<DateTime<Utc>>,
    pub guardian_name: Option<String>,
    pub guardian_document: Option<String>,
    pub guardian_phone: Option<String>,
    pub course_id: Option<String>,
    pub group_id: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Student {
    /// Create a new student
    pub fn new(
        id: String,
        user_id: String,
        first_name: String,
        last_name: String,
        document_type: String,
        document_number: String,
        email: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            user_id,
            first_name,
            last_name,
            document_type,
            document_number,
            email,
            phone: None,
            address: None,
            birth_date: None,
            guardian_name: None,
            guardian_document: None,
            guardian_phone: None,
            course_id: None,
            group_id: None,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get student's full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if student is minor (less than 18 years old)
    pub fn is_minor(&self) -> bool {
        if let Some(birth_date) = self.birth_date {
            let now = Utc::now();
            let age = now.signed_duration_since(birth_date).num_days() / 365;
            return age < 18;
        }
        false
    }

    /// Update student information
    pub fn update(
        &mut self,
        first_name: Option<String>,
        last_name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        address: Option<String>,
        birth_date: Option<DateTime<Utc>>,
        guardian_name: Option<String>,
        guardian_document: Option<String>,
        guardian_phone: Option<String>,
        course_id: Option<String>,
        group_id: Option<String>,
    ) {
        if let Some(f) = first_name {
            self.first_name = f;
        }
        if let Some(l) = last_name {
            self.last_name = l;
        }
        if let Some(e) = email {
            self.email = e;
        }
        if let Some(p) = phone {
            self.phone = Some(p);
        }
        if let Some(a) = address {
            self.address = Some(a);
        }
        if let Some(b) = birth_date {
            self.birth_date = Some(b);
        }
        if let Some(g) = guardian_name {
            self.guardian_name = Some(g);
        }
        if let Some(d) = guardian_document {
            self.guardian_document = Some(d);
        }
        if let Some(t) = guardian_phone {
            self.guardian_phone = Some(t);
        }
        if course_id.is_some() {
            self.course_id = course_id;
        }
        if group_id.is_some() {
            self.group_id = group_id;
        }
        self.updated_at = Utc::now();
    }

    /// Deactivate student (soft delete)
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }
}
