//! Academix - Desktop Academic Management System
//!
//! A Tauri 2 + React 19 desktop application using Hexagonal Architecture.

pub mod application;
pub mod commands;
pub mod domain;
pub mod env_loader;
pub mod infrastructure;

use application::use_cases::{
    AttendanceService, CourseService, GroupService, InvoiceService,
    PaymentService, SettingsService, StudentService, UserService,
};
use commands::accounting::{create_entry, get_entry, get_accounting_summary, list_entries, delete_entry};
use commands::attendance::{
    count_group_absences, count_student_absences, create_attendance, delete_attendance,
    get_attendance, get_group_attendance_stats, list_attendance_by_group_date,
    list_attendance_by_student, list_attendances, update_attendance,
};
use commands::auth::{change_password, login, logout, update_profile, AppState as AuthAppState};
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
use commands::users::{create_user, delete_user, get_user, list_users, list_users_by_role, update_user};
use infrastructure::database::SqlitePool;
use infrastructure::repositories::{
    SqliteAttendanceRepository, SqliteCourseRepository, SqliteGroupRepository,
    SqliteInvoiceRepository, SqliteInvoiceLineRepository,
    SqlitePaymentRepository, SqliteSettingsRepository, SqliteStudentRepository,
    SqliteUserRepository, SqliteAccountingEntryRepository,
};
use std::path::PathBuf;
use std::sync::Arc;

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

/// Initialize database pool and run migrations
fn init_database() -> SqlitePool {
    let db_path = get_db_path();
    println!("Database path: {:?}", db_path);

    let pool = SqlitePool::new(db_path.clone()).expect("Failed to create database pool");

    let conn_ref = pool.connection();
    let conn = conn_ref.lock().unwrap();

    // Helper to run migration with idempotent error handling
    macro_rules! run_migration {
        ($conn:expr, $name:expr, $sql:expr) => {
            match $conn.execute_batch($sql) {
                Ok(_) => println!("Migration {} applied", $name),
                Err(e) => {
                    let err_str = format!("{}", e);
                    if err_str.contains("duplicate column name") || err_str.contains("table students has no column") {
                        println!("Migration {} already applied (idempotent)", $name);
                    } else {
                        eprintln!("WARNING running migration {}: {}", $name, e);
                    }
                }
            }
        };
    }

    // Run initial schema migration
    let migration_sql = include_str!("../migrations/001_initial_schema.sql");
    conn.execute_batch(migration_sql)
        .expect("Failed to run initial schema migration");

    // Run migrations 003-009 (existing)
    run_migration!(conn, "003", include_str!("../migrations/003_add_guardian_and_schedule_fields.sql"));
    run_migration!(conn, "004", include_str!("../migrations/004_add_student_enrollment_columns.sql"));
    run_migration!(conn, "005", include_str!("../migrations/005_add_course_price.sql"));
    run_migration!(conn, "006", include_str!("../migrations/006_add_group_schedule_fields.sql"));
    run_migration!(conn, "007", include_str!("../migrations/007_add_start_date_to_groups.sql"));
    run_migration!(conn, "008", include_str!("../migrations/008_fix_groups_table_schema.sql"));
    run_migration!(conn, "009", include_str!("../migrations/009_make_payments_due_date_nullable.sql"));

    // Run migration 010 - original accounting schema
    run_migration!(conn, "010", include_str!("../migrations/010_accounting_schema.sql"));

    // Run migration 011 - accounting seed
    run_migration!(conn, "011", include_str!("../migrations/011_accounting_seed.sql"));

    // Run migration 012 - liabilities and equity
    run_migration!(conn, "012", include_str!("../migrations/012_liabilities_equity_schema.sql"));

    // Run migration 013 - fixed assets
    run_migration!(conn, "013", include_str!("../migrations/013_fixed_assets_schema.sql"));

    // Run migration 014 - fixed assets accounts
    run_migration!(conn, "014", include_str!("../migrations/014_fixed_assets_accounts.sql"));

    // Run migration 015 - pasivos accounts
    run_migration!(conn, "015", include_str!("../migrations/015_pasivos_accounts.sql"));

    // Run migration 016 - simplify accounting schema
    run_migration!(conn, "016", include_str!("../migrations/016_simplify_accounting_schema.sql"));

    // Run migration 017 - add app settings table
    run_migration!(conn, "017", include_str!("../migrations/017_add_app_settings.sql"));

    // Run migration 018 - add class_duration and skipped_dates to groups
    run_migration!(conn, "018", include_str!("../migrations/018_add_class_duration_and_skipped_dates.sql"));

    println!("Database initialized successfully");
    
    drop(conn);
    
    pool
}

/// Seed admin user if not exists
fn seed_admin_user(pool: &SqlitePool) {
    use crate::application::ports::UserRepository;
    use crate::domain::entities::user::{Role, User};
    use crate::domain::value_objects::Email;
    use crate::env_loader::{get_env_var, is_production};

    let repo = SqliteUserRepository::new(Arc::new(pool.clone()));

    let admin_email = match get_env_var("ADMIN_EMAIL", Some("admin@academix.com")) {
        Ok(email) => {
            if is_production() {
                println!("[ENV] Using ADMIN_EMAIL from environment: {}", email);
            } else {
                eprintln!(
                    "[WARNING] Using default admin email. Set ADMIN_EMAIL env var for production."
                );
            }
            email
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to get admin email: {}", e);
            return;
        }
    };

    let admin_password_hash = match get_env_var(
        "ADMIN_PASSWORD_HASH",
        Some("$2b$12$gghetCr2w7EqfgK5u8jMru4Malw8kQZcXMUQfp2dwOsac2xlo5gYy"),
    ) {
        Ok(hash) => {
            if is_production() {
                println!("[ENV] Using ADMIN_PASSWORD_HASH from environment");
            } else {
                eprintln!(
                    "[WARNING] Using default admin password hash. Set ADMIN_PASSWORD_HASH env var for production."
                );
            }
            hash
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to get admin password hash: {}", e);
            return;
        }
    };

    let email_value = match Email::new(admin_email.clone()) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[ERROR] Invalid admin email format: {}", e);
            return;
        }
    };

    if repo.exists_by_email(&email_value).unwrap_or(false) {
        println!("Admin user already exists");
        return;
    }

    let admin = User::new(
        "admin-1".to_string(),
        admin_email,
        admin_password_hash,
        "Luifer Admin".to_string(),
        Role::Admin,
    );

    if let Err(e) = repo.save(&admin) {
        eprintln!("Failed to seed admin user: {}", e);
    } else {
        println!("Admin user seeded successfully");
    }
}

/// Create service states with SQLite repositories
fn create_service_states(
    pool: Arc<SqlitePool>,
) -> (
    UserService<SqliteUserRepository>,
    StudentService<SqliteStudentRepository, SqliteGroupRepository>,
    CourseService<SqliteCourseRepository>,
    GroupService<SqliteGroupRepository, SqliteCourseRepository>,
    PaymentService<SqlitePaymentRepository, SqliteGroupRepository, SqliteCourseRepository>,
    AttendanceService<SqliteAttendanceRepository>,
    InvoiceService<SqliteInvoiceRepository, SqliteInvoiceLineRepository>,
    crate::application::use_cases::AccountingService<SqliteAccountingEntryRepository>,
    SettingsService<SqliteSettingsRepository>,
) {
    let user_repo = SqliteUserRepository::new(Arc::clone(&pool));
    let student_repo = SqliteStudentRepository::new(Arc::clone(&pool));
    let course_repo = SqliteCourseRepository::new(Arc::clone(&pool));
    let group_repo = SqliteGroupRepository::new(Arc::clone(&pool));
    let payment_repo = SqlitePaymentRepository::new(Arc::clone(&pool));
    let attendance_repo = SqliteAttendanceRepository::new(Arc::clone(&pool));
    let invoice_repo = SqliteInvoiceRepository::new(Arc::clone(&pool));
    let invoice_line_repo = SqliteInvoiceLineRepository::new(Arc::clone(&pool));
    let accounting_entry_repo = SqliteAccountingEntryRepository::new(Arc::clone(&pool));
    let settings_repo = SqliteSettingsRepository::new(Arc::clone(&pool));
    let accounting_service = crate::application::use_cases::AccountingService::new(
        accounting_entry_repo.clone(),
    );

    (
        UserService::new(user_repo),
        StudentService::new(student_repo, SqliteGroupRepository::new(Arc::clone(&pool))),
        CourseService::new(SqliteCourseRepository::new(Arc::clone(&pool))),
        GroupService::new(group_repo.clone(), SqliteCourseRepository::new(Arc::clone(&pool))),
        PaymentService::new(payment_repo, group_repo, course_repo),
        AttendanceService::new(attendance_repo),
        InvoiceService::new(invoice_repo, invoice_line_repo),
        accounting_service,
        SettingsService::new(settings_repo),
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database with migrations
    println!("Initializing database...");
    let pool = init_database();

    // Seed admin user
    seed_admin_user(&pool);

    // Create all service states with SQLite repositories
    let pool = Arc::new(pool);
    let (
        user_service,
        student_service,
        course_service,
        group_service,
        payment_service,
        attendance_service,
        invoice_service,
        accounting_service,
        settings_service,
    ) = create_service_states(Arc::clone(&pool));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AuthAppState::new(Arc::clone(&pool)))
        .manage(user_service)
        .manage(student_service)
        .manage(course_service)
        .manage(group_service)
        .manage(payment_service)
        .manage(attendance_service)
        .manage(invoice_service)
        .manage(accounting_service)
        .manage(settings_service)
        .invoke_handler(tauri::generate_handler![
            // Health check
            health,
            // Public registration (no auth required)
            register_user,
            // Auth commands
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
