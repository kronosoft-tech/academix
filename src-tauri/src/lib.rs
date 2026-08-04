//! Academix - Desktop Academic Management System
//!
//! A Tauri 2 + React 19 desktop application using Hexagonal Architecture.
//! Phase 4: Hard cutover from local SQLite to Turso for authentication.
//! Login resolves via ControlPlane → ConnectionManager → user's Turso DB.

pub mod application;
pub mod commands;
pub mod domain;
pub mod env_loader;
pub mod infrastructure;

use application::use_cases::{
    AttendanceService, CourseService, GroupService, InvoiceService, PaymentService,
    SettingsService, StudentService, UserService,
};
use commands::accounting::{
    create_entry, delete_entry, get_accounting_summary, get_entry, list_entries,
};
use commands::admin::list_client_databases;
use commands::attendance::{
    count_group_absences, count_student_absences, create_attendance, delete_attendance,
    get_attendance, get_group_attendance_stats, list_attendance_by_group_date,
    list_attendance_by_student, list_attendances, update_attendance,
};
use commands::auth::{change_password, login, logout, update_profile, AppState as TursoAppState};
use commands::base::health;
use commands::courses::{
    archive_course, create_course, delete_course, get_course, hard_delete_course,
    list_archived_courses, list_courses, restore_course, update_course,
};
use commands::groups::{create_group, delete_group, get_group, list_groups, update_group};
use commands::invoices::{
    cancel_invoice, create_invoice, get_invoice, get_invoice_summary, list_invoices,
    register_payment,
};
use commands::payments::{
    create_payment, delete_payment, get_all_students_payment_summary, get_payment,
    get_student_payment_status, list_payments, list_payments_by_student, update_payment,
};
use commands::register::register_user;
use commands::settings::{get_absence_threshold, set_absence_threshold};
use commands::students::{
    create_student, delete_student, get_student, list_students, update_student,
};
use commands::updater::{check_for_update, get_update_check_interval, set_update_check_interval};
use commands::users::{
    create_user, delete_user, get_user, list_users, list_users_by_role, update_user,
};
use env_loader::load_turso_config;
use infrastructure::local_db;
use infrastructure::repositories::{
    MemoryBackedAccountingEntryRepository, MemoryBackedAttendanceRepository,
    MemoryBackedCourseRepository, MemoryBackedGroupRepository, MemoryBackedInvoiceLineRepository,
    MemoryBackedInvoiceRepository, MemoryBackedPaymentRepository, MemoryBackedSettingsRepository,
    MemoryBackedStudentRepository, MemoryBackedUserRepository,
};
use infrastructure::turso::connection_manager::ConnectionManager;
use infrastructure::turso::control_plane::ControlPlaneRepository;
use infrastructure::turso::flush_timer::start_flush_timer;
use infrastructure::turso::memory_buffer::MemoryBuffer;
use infrastructure::turso::provisioning::TursoProvisioningService;
use infrastructure::turso::session::CurrentSession;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Get the database path in the app data directory
fn get_db_path() -> PathBuf {
    let app_data = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("academix");

    std::fs::create_dir_all(&app_data).ok();
    let db_path = app_data.join("academix.db");
    eprintln!("[DB PATH] Using database at: {:?}", db_path);
    db_path
}

/// Run all local SQLite migrations (for backward-compatible read operations).
/// Phase 4: Old commands still read from local SQLite until fully migrated.
///
/// Uses a `_schema_migrations` tracking table so each migration runs only once.
/// Existing databases are treated as fully migrated up to 018 to avoid re-running
/// non-idempotent schema migrations (010-016).
async fn run_local_migrations() {
    let db_path = get_db_path();
    println!("Database path: {:?}", db_path);

    let db = match libsql::Builder::new_local(db_path).build().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[FATAL] Cannot open database: {}", e);
            return;
        }
    };

    local_db::init(db);

    let conn = match local_db::get_db().connect() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[FATAL] Cannot connect to database: {}", e);
            return;
        }
    };

    // --- Migration tracking setup ---
    // Create the tracking table if it doesn't exist
    if let Err(e) = conn
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS _schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        )
        .await
    {
        eprintln!("[WARN] Could not create _schema_migrations: {}", e);
    }

    // Load already-applied migrations
    let mut applied: Vec<String> = Vec::new();
    if let Ok(mut rows) = conn
        .query(
            "SELECT version FROM _schema_migrations ORDER BY version",
            (),
        )
        .await
    {
        loop {
            match rows.next().await {
                Ok(Some(row)) => {
                    if let Ok(ver) = row.get_str(0) {
                        applied.push(ver.to_string());
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
    }

    // Track if this is a legacy database (no tracking table before)
    let is_legacy = applied.is_empty();

    // --- Legacy database migration: seed tracking for already-applied migrations ---
    // For existing databases that were running before this tracking was added,
    // we seed the tracking table with all existing migrations (001-018).
    //
    // This avoids re-running non-idempotent migrations (010-016) that left the
    // database in a partially-migrated state.
    if is_legacy {
        println!("First run with migration tracking — seeding applied state for existing database (001-018)");

        // Clean up leftover table from migration 016 partial run
        conn.execute_batch("DROP TABLE IF EXISTS accounting_entries_new;")
            .await
            .ok();

        let existing: [&str; 17] = [
            "001", "003", "004", "005", "006", "007", "008", "009", "010", "011", "012", "013",
            "014", "015", "016", "017", "018",
        ];
        for ver in &existing {
            conn.execute_batch(&format!(
                "INSERT OR IGNORE INTO _schema_migrations (version) VALUES ('{}')",
                ver
            ))
            .await
            .ok();
        }
        println!(
            "Legacy database seeded — {} migrations tracked",
            existing.len()
        );
        println!("Local database initialized successfully");
        return;
    }

    // --- Helper: run a migration only if not already applied ---
    macro_rules! run_migration {
        ($name:expr, $sql:expr) => {
            if applied.contains(&$name.to_string()) {
                println!("Migration {} already applied", $name);
            } else {
                match conn.execute_batch($sql).await {
                    Ok(_) => {
                        let _ = conn
                            .execute_batch(&format!(
                                "INSERT INTO _schema_migrations (version) VALUES ('{}')",
                                $name
                            ))
                            .await;
                        println!("Migration {} applied", $name);
                    }
                    Err(e) => {
                        eprintln!("WARNING running migration {}: {}", $name, e);
                    }
                }
            }
        };
    }

    // --- Run migrations in order ---

    // 001 — initial schema (always safe: CREATE TABLE IF NOT EXISTS)
    if applied.contains(&"001".to_string()) {
        println!("Migration 001 already applied");
    } else {
        let sql = include_str!("../migrations/001_initial_schema.sql");
        match conn.execute_batch(sql).await {
            Ok(_) => {
                let _ = conn
                    .execute_batch("INSERT INTO _schema_migrations (version) VALUES ('001')")
                    .await;
                println!("Migration 001 applied");
            }
            Err(e) => eprintln!("WARNING running migration 001: {}", e),
        }
    }

    // 003-018 via tracking table
    run_migration!(
        "003",
        include_str!("../migrations/003_add_guardian_and_schedule_fields.sql")
    );
    run_migration!(
        "004",
        include_str!("../migrations/004_add_student_enrollment_columns.sql")
    );
    run_migration!(
        "005",
        include_str!("../migrations/005_add_course_price.sql")
    );
    run_migration!(
        "006",
        include_str!("../migrations/006_add_group_schedule_fields.sql")
    );
    run_migration!(
        "007",
        include_str!("../migrations/007_add_start_date_to_groups.sql")
    );
    run_migration!(
        "008",
        include_str!("../migrations/008_fix_groups_table_schema.sql")
    );
    run_migration!(
        "009",
        include_str!("../migrations/009_make_payments_due_date_nullable.sql")
    );
    run_migration!(
        "010",
        include_str!("../migrations/010_accounting_schema.sql")
    );
    run_migration!("011", include_str!("../migrations/011_accounting_seed.sql"));
    run_migration!(
        "012",
        include_str!("../migrations/012_liabilities_equity_schema.sql")
    );
    run_migration!(
        "013",
        include_str!("../migrations/013_fixed_assets_schema.sql")
    );
    run_migration!(
        "014",
        include_str!("../migrations/014_fixed_assets_accounts.sql")
    );
    run_migration!(
        "015",
        include_str!("../migrations/015_pasivos_accounts.sql")
    );
    run_migration!(
        "016",
        include_str!("../migrations/016_simplify_accounting_schema.sql")
    );
    run_migration!(
        "017",
        include_str!("../migrations/017_add_app_settings.sql")
    );
    run_migration!(
        "018",
        include_str!("../migrations/018_add_class_duration_and_skipped_dates.sql")
    );
    run_migration!(
        "019",
        include_str!("../migrations/019_add_payment_type.sql")
    );

    println!("Local database initialized successfully");
}

/// Seed admin/superadmin in control plane Turso DB on startup.
/// Also provisions a per-user Turso database for the admin so they can log in.
#[allow(dead_code)]
async fn seed_control_plane_admin(
    cp: &ControlPlaneRepository,
    prov: Option<&Arc<TursoProvisioningService>>,
    config: Option<&env_loader::TursoConfig>,
) {
    use env_loader::{get_env_var, is_production};

    let admin_email = get_env_var("ADMIN_EMAIL", Some("admin@academix.com"))
        .unwrap_or_else(|_| "admin@academix.com".to_string());
    let admin_password_hash = get_env_var(
        "ADMIN_PASSWORD_HASH",
        Some("$2b$12$gghetCr2w7EqfgK5u8jMru4Malw8kQZcXMUQfp2dwOsac2xlo5gYy"),
    )
    .unwrap_or_else(|_| "$2b$12$gghetCr2w7EqfgK5u8jMru4Malw8kQZcXMUQfp2dwOsac2xlo5gYy".to_string());

    if !is_production() {
        eprintln!("[WARNING] Using default admin credentials. Set ADMIN_EMAIL and ADMIN_PASSWORD_HASH env vars for production.");
    }

    // Check if admin already exists
    let user_id = match cp.find_user_by_email(&admin_email).await {
        Ok(Some(user)) => {
            println!("Superadmin already exists in control plane");
            user.id
        }
        Ok(None) => {
            let user_id = "admin-1".to_string();
            let now = chrono::Utc::now().to_rfc3339();
            let superadmin = infrastructure::turso::control_plane::UserRow {
                id: user_id.clone(),
                email: admin_email.clone(),
                password_hash: admin_password_hash.clone(),
                name: "Luifer Admin".to_string(),
                role: "Admin".to_string(),
                is_active: true,
                created_at: now.clone(),
                updated_at: now.clone(),
            };
            if let Err(e) = cp.save_user(&superadmin).await {
                eprintln!("[ERROR] Failed to seed superadmin: {}", e);
                return;
            }
            println!("Superadmin seeded in control plane");
            user_id
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to check superadmin: {}", e);
            return;
        }
    };

    // Provision per-user Turso DB for admin if provisioning is available
    if let (Some(prov), Some(config)) = (prov, config) {
        if cp.find_by_user_id(&user_id).await.ok().flatten().is_none() {
            eprintln!("[ADMIN] Provisioning per-user Turso database for admin...");
            let slug = format!("academix-admin-{}", user_id);
            match prov.create_database(&slug, Some(&config.turso_group)).await {
                Ok(db_info) => {
                    let db_name = format!("libsql://{}", db_info.hostname);
                    match prov.create_auth_token(&db_name).await {
                        Ok(token) => {
                            let now = chrono::Utc::now().to_rfc3339();
                            let mapping = infrastructure::turso::control_plane::UserDbMapping {
                                user_id: user_id.clone(),
                                email: admin_email.clone(),
                                academy_name: "academix".to_string(),
                                db_url: db_name.clone(),
                                db_token: token,
                                org: config.turso_org.clone(),
                                created_at: now.clone(),
                            };
                            if let Err(e) = cp.save_user_db(&mapping).await {
                                eprintln!("[ADMIN] Failed to save DB mapping: {}", e);
                                return;
                            }
                            println!("[ADMIN] Per-user Turso DB provisioned");

                            // Seed admin user record in the new per-user DB
                            let db = match libsql::Builder::new_remote(
                                db_name.clone(),
                                mapping.db_token.clone(),
                            )
                            .build()
                            .await
                            {
                                Ok(db) => db,
                                Err(e) => {
                                    eprintln!("[ADMIN] Failed to connect to new DB: {}", e);
                                    return;
                                }
                            };
                            let conn = db.connect().unwrap();
                            let _ = conn.execute_batch(include_str!(
                                "../migrations/001_initial_schema.sql"
                            ));
                            let _ = conn
                                .execute(
                                    "INSERT OR IGNORE INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
                                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                                    libsql::params![
                                        user_id.clone(),
                                        admin_email.clone(),
                                        admin_password_hash.clone(),
                                        "Luifer Admin".to_string(),
                                        "Admin".to_string(),
                                        1,
                                        now.clone(),
                                        now.clone(),
                                    ],
                                )
                                .await;
                            println!("[ADMIN] Admin user seeded in per-user Turso DB");
                        }
                        Err(e) => eprintln!("[ADMIN] Failed to create auth token: {}", e),
                    }
                }
                Err(e) => eprintln!("[ADMIN] Failed to create Turso DB: {}", e),
            }
        }
    }
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // Phase 4: Initialize local SQLite for backward-compatible read operations
    // Old command handlers still read from local DB until Phase 5 full migration
    println!("Initializing local database for backward compatibility...");
    run_local_migrations().await;

    // Initialize Turso services (control plane + provisioning)
    println!("Loading Turso configuration...");
    let turso_config = load_turso_config();

    let (
        config,
        control_plane,
        provisioning_service,
        connection_manager,
        memory_buffer,
        flush_timer_sender,
    ) = match turso_config {
        Ok(config) => {
            println!("[TURSO] Turso config loaded, initializing services...");

            let prov = Arc::new(TursoProvisioningService::new(
                config.turso_api_token.clone(),
                config.turso_org.clone(),
            ));

            let cp = match ControlPlaneRepository::new(
                &config.control_plane_db_url,
                &config.control_plane_db_token,
            )
            .await
            {
                Ok(cp) => {
                    if let Err(e) = cp.ensure_schema().await {
                        eprintln!("[TURSO] Failed to ensure control plane schema: {}", e);
                    } else {
                        println!("[TURSO] Control plane schema ready");
                    }
                    // seed_control_plane_admin is now disabled — user registers themselves
                    // seed_control_plane_admin(&cp, Some(&prov), Some(&config)).await;
                    (Some(Arc::new(cp)), Some(prov))
                }
                Err(e) => {
                    eprintln!("[TURSO] Failed to connect to control plane: {}", e);
                    eprintln!(
                        "[TURSO] Control plane features disabled. Set CONTROL_PLANE_DB_URL \
                         and CONTROL_PLANE_DB_TOKEN to enable."
                    );
                    (None, Some(prov))
                }
            };

            let (control_plane, provisioning) = cp;

            let connection_manager = Arc::new(Mutex::new(ConnectionManager::new()));
            let memory_buffer = Arc::new(Mutex::new(MemoryBuffer::new()));
            let flush_timer_sender = control_plane.as_ref().map(|cp_arc| {
                start_flush_timer(
                    Arc::clone(&memory_buffer),
                    Arc::clone(&connection_manager),
                    Arc::clone(cp_arc),
                )
            });

            (
                Some(config),
                control_plane,
                provisioning,
                connection_manager,
                memory_buffer,
                flush_timer_sender,
            )
        }
        Err(e) => {
            eprintln!("[TURSO] Turso config not available: {}", e);
            eprintln!("[TURSO] Turso features disabled. Set env vars to enable (see README).");
            let cm = Arc::new(Mutex::new(ConnectionManager::new()));
            let mb = Arc::new(Mutex::new(MemoryBuffer::new()));
            (None, None, None, cm, mb, None)
        }
    };

    // Phase 5: MemoryBuffer-backed repositories (reads from Turso per-user DBs)
    let session = Arc::new(Mutex::new(CurrentSession { user_id: None }));

    let user_service = UserService::new(MemoryBackedUserRepository::new(
        Arc::clone(&connection_manager),
        Arc::clone(&memory_buffer),
        Arc::clone(&session),
    ));
    let student_service = StudentService::new(
        MemoryBackedStudentRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
        MemoryBackedGroupRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
    );
    let course_service = CourseService::new(MemoryBackedCourseRepository::new(
        Arc::clone(&connection_manager),
        Arc::clone(&memory_buffer),
        Arc::clone(&session),
    ));
    let group_service = GroupService::new(
        MemoryBackedGroupRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
        MemoryBackedCourseRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
    );
    let payment_service = PaymentService::new(
        MemoryBackedPaymentRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
        MemoryBackedGroupRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
        MemoryBackedCourseRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
    );
    let attendance_service = AttendanceService::new(MemoryBackedAttendanceRepository::new(
        Arc::clone(&connection_manager),
        Arc::clone(&memory_buffer),
        Arc::clone(&session),
    ));
    let invoice_service = InvoiceService::new(
        MemoryBackedInvoiceRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
        MemoryBackedInvoiceLineRepository::new(
            Arc::clone(&connection_manager),
            Arc::clone(&memory_buffer),
            Arc::clone(&session),
        ),
    );
    let accounting_repo = MemoryBackedAccountingEntryRepository::new(
        Arc::clone(&connection_manager),
        Arc::clone(&memory_buffer),
        Arc::clone(&session),
    );
    let accounting_service =
        crate::application::use_cases::AccountingService::new(accounting_repo.clone());
    let settings_service = SettingsService::new(MemoryBackedSettingsRepository::new(
        Arc::clone(&connection_manager),
        Arc::clone(&memory_buffer),
        Arc::clone(&session),
    ));

    // Build the Turso AppState
    let turso_state = TursoAppState {
        connection_manager: Arc::clone(&connection_manager),
        memory_buffer: Arc::clone(&memory_buffer),
        control_plane: control_plane.as_ref().map(Arc::clone),
        flush_timer_sender,
        session: Arc::clone(&session),
        turso_config: config.clone(),
    };

    // Build Tauri app
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Register Turso AppState (required for auth commands)
        .manage(turso_state)
        // Register old service states for backward compatibility
        .manage(user_service)
        .manage(student_service)
        .manage(course_service)
        .manage(group_service)
        .manage(payment_service)
        .manage(attendance_service)
        .manage(invoice_service)
        .manage(accounting_service)
        .manage(accounting_repo)
        .manage(settings_service)
        // Register optional Turso services (None when not configured)
        .manage(control_plane)
        .manage(provisioning_service)
        // Spawn update scheduler on app startup
        .setup(|app| {
            let handle = app.handle().clone();
            infrastructure::updater::UpdateScheduler::start(handle, 4);
            Ok(())
        })
        // Register all command handlers
        .invoke_handler(tauri::generate_handler![
            // Health check
            health,
            // Public registration (no auth required)
            register_user,
            // Auth commands (Turso-backed)
            login,
            logout,
            update_profile,
            change_password,
            // User commands
            create_user,
            get_user,
            list_users,
            list_users_by_role,
            update_user,
            delete_user,
            // Student commands
            create_student,
            get_student,
            list_students,
            update_student,
            delete_student,
            // Course commands
            create_course,
            get_course,
            list_courses,
            update_course,
            delete_course,
            archive_course,
            restore_course,
            hard_delete_course,
            list_archived_courses,
            // Group commands
            create_group,
            get_group,
            list_groups,
            update_group,
            delete_group,
            // Payment commands
            create_payment,
            get_payment,
            list_payments,
            list_payments_by_student,
            update_payment,
            delete_payment,
            get_student_payment_status,
            get_all_students_payment_summary,
            // Attendance commands
            create_attendance,
            get_attendance,
            list_attendances,
            list_attendance_by_group_date,
            list_attendance_by_student,
            update_attendance,
            delete_attendance,
            get_group_attendance_stats,
            count_student_absences,
            count_group_absences,
            // Accounting commands (simplified)
            create_entry,
            get_entry,
            list_entries,
            delete_entry,
            get_accounting_summary,
            // Invoice commands
            create_invoice,
            get_invoice,
            list_invoices,
            register_payment,
            cancel_invoice,
            get_invoice_summary,
            // Settings commands
            get_absence_threshold,
            set_absence_threshold,
            // Admin commands (superadmin)
            list_client_databases,
            // Updater commands
            check_for_update,
            get_update_check_interval,
            set_update_check_interval,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
