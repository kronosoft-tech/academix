//! Registration Use Case
//!
//! Handles user registration with validation, password hashing,
//! and optional Turso database provisioning.

use crate::application::dto::{RegisterUserRequest, RegisterUserResponse};
use crate::application::errors::ApplicationError;
use crate::application::ports::UserRepository;
use crate::domain::entities::{Role, User};
use crate::domain::value_objects::{Email, Password};
use crate::infrastructure::password;
use crate::infrastructure::turso::connection_manager::run_migrations_on_db;
use crate::infrastructure::turso::control_plane::{ControlPlaneRepository, UserDbMapping};
use crate::infrastructure::turso::provisioning::{generate_db_slug, TursoProvisioningService};
use std::sync::Arc;
use uuid::Uuid;

/// Registration use case
pub struct RegisterUserUseCase<R: UserRepository> {
    user_repository: R,
    control_plane: Option<Arc<ControlPlaneRepository>>,
    provisioning: Option<Arc<TursoProvisioningService>>,
    org: Option<String>,
}

impl<R: UserRepository> RegisterUserUseCase<R> {
    pub fn new(
        user_repository: R,
        control_plane: Option<Arc<ControlPlaneRepository>>,
        provisioning: Option<Arc<TursoProvisioningService>>,
    ) -> Self {
        Self {
            user_repository,
            control_plane,
            provisioning,
            org: None,
        }
    }

    /// Set the Turso organization slug for per-user database provisioning.
    pub fn with_org(mut self, org: String) -> Self {
        self.org = Some(org);
        self
    }

    /// Register a new user as the academy owner (Admin role).
    ///
    /// If Turso services are configured, also provisions a per-user database
    /// and saves the mapping to the control plane.
    /// The registering user is always Admin — they are creating their own academy.
    /// Additional users (employees, professors) are added by the admin inside the app.
    pub async fn execute(
        &self,
        request: RegisterUserRequest,
    ) -> Result<RegisterUserResponse, ApplicationError> {
        // Clone values needed before consuming request
        let email_str = request.email.clone();
        let academy_name = request.academy_name.clone();
        let name = request.name.clone();

        // Validate email format
        let email = Email::new(&email_str).map_err(|e| ApplicationError::Validation(e))?;

        // Validate password minimum 8 characters
        Password::validate_strength(&request.password)
            .map_err(|e| ApplicationError::Validation(e))?;

        // Check if email already exists — via control plane when Turso is configured,
        // otherwise fall back to the user repository
        if let Some(cp) = &self.control_plane {
            if cp
                .find_by_email(&email_str)
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!("control plane error: {}", e))
                })?
                .is_some()
            {
                return Err(ApplicationError::Conflict(
                    "Email already registered".to_string(),
                ));
            }
        } else if self.user_repository.exists_by_email(&email).await? {
            return Err(ApplicationError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        // Hash password
        let password_hash = password::hash_password(&request.password)
            .map_err(|e| ApplicationError::Infrastructure(e.to_string()))?;

        // Generate user ID
        let user_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();

        // If Turso is configured, provision database and save mapping
        if let (Some(cp), Some(prov)) = (&self.control_plane, &self.provisioning) {
            let slug = generate_db_slug(&academy_name);

            println!(
                "[REGISTER] Step 1: Creating Turso database... slug={}, org={:?}",
                slug, self.org
            );
            let db_info = prov
                .create_database(&slug, self.org.as_deref())
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "turso: failed to create database: {}",
                        e
                    ))
                })?;

            println!(
                "[REGISTER] Step 2: DB created! hostname={}. Creating auth token...",
                db_info.hostname
            );
            let token = prov.create_auth_token(&slug).await.map_err(|e| {
                ApplicationError::Infrastructure(format!(
                    "turso: failed to create auth token: {}",
                    e
                ))
            })?;

            let now = chrono::Utc::now().to_rfc3339();

            println!(
                "[REGISTER] Step 3: Auth token created. Connecting to libsql://{}...",
                db_info.hostname
            );
            // Connect to the newly created Turso DB and initialize it
            let db_url = format!("libsql://{}", db_info.hostname);
            let new_db = libsql::Builder::new_remote(db_url.clone(), token.clone())
                .build()
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "Failed to connect to new Turso DB: {}",
                        e
                    ))
                })?;

            println!("[REGISTER] Step 4: Connected to Turso DB. Running migrations...");
            // Run all 18 migrations on the new DB
            run_migrations_on_db(&new_db).await.map_err(|e| {
                ApplicationError::Infrastructure(format!(
                    "Failed to run migrations on new Turso DB: {}",
                    e
                ))
            })?;

            println!("[REGISTER] Step 5: Migrations done. Saving user as Admin (academy owner)...");
            // Save user record directly to the new Turso DB.
            // The registering user is ALWAYS Admin — they are the academy owner.
            // Additional users (employees, professors) are added later by the admin.
            {
                let conn = new_db.connect().map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "Failed to connect to new DB for user insert: {}",
                        e
                    ))
                })?;

                conn.execute(
                    "INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    libsql::params![
                        user_id.clone(),
                        email_str.clone(),
                        password_hash.clone(),
                        name.clone(),
                        "Admin", // Academy owner is always Admin (capitalized per DB CHECK constraint)
                        1,       // is_active = true
                        now.clone(),
                        now.clone(),
                    ],
                )
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "Failed to save user in Turso DB: {}",
                        e
                    ))
                })?;

                println!(
                    "[TURSO] User '{}' saved as Admin to new Turso DB '{}'",
                    email_str, db_info.hostname
                );
            }

            let mapping = UserDbMapping {
                user_id: user_id.clone(),
                email: email_str.clone(),
                academy_name: academy_name.clone(),
                db_url,
                db_token: token,
                org: prov.org().to_string(),
                created_at: now,
            };

            // Save user→DB mapping to control plane
            if let Err(e) = cp.save_user_db(&mapping).await {
                eprintln!("[TURSO] Failed to save user DB mapping (non-fatal): {}", e);
            }

            // Save user credentials to control plane users table (for web login)
            {
                use crate::infrastructure::turso::control_plane::UserRow;
                let now_cp = chrono::Utc::now().to_rfc3339();
                let user_row = UserRow {
                    id: user_id.clone(),
                    email: email_str.clone(),
                    password_hash: password_hash.clone(),
                    name: name.clone(),
                    role: "user".to_string(),
                    is_active: true,
                    created_at: now_cp.clone(),
                    updated_at: now_cp,
                };
                if let Err(e) = cp.save_user(&user_row).await {
                    eprintln!(
                        "[TURSO] Failed to save user to control plane (non-fatal): {}",
                        e
                    );
                }
            }

            // Register the connection in ConnectionManager so login can find it
            // The ConnectionManager is not available here (it's owned by AppState),
            // but login will resolve via control plane → init_connection lazily.
        }

        // Create user with Admin role — the registering user owns this academy.
        // Employees/professors are added later via the admin panel inside the app.
        let user = User::new(user_id, email_str, password_hash, name, Role::Admin);

        // When Turso is configured, the user was already saved directly to their DB above.
        // Only save via the repository for the non-Turso fallback path.
        if self.control_plane.is_none() {
            self.user_repository.save(&user).await?;
        }

        Ok(RegisterUserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
        })
    }
}
