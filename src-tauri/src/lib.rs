//! Academix - Desktop Academic Management System
//!
//! A Tauri 2 + React 19 desktop application using Hexagonal Architecture.

pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

use application::use_cases::{
    AttendanceService, CourseService, GroupService, PaymentService, StudentService, UserService,
};
use commands::accounting::{
    create_entry, get_entry, list_entries, get_trial_balance, get_income_statement,
    list_accounts, get_account_tree, get_accounting_summary,
};
use commands::attendance::{
    create_attendance, delete_attendance, get_attendance, get_group_attendance_stats,
    list_attendance_by_group_date, list_attendance_by_student, list_attendances, update_attendance,
};
use commands::auth::{login, logout, AppState as AuthAppState};
use commands::base::health;
use commands::courses::{
    archive_course, create_course, delete_course, get_course, hard_delete_course,
    list_archived_courses, list_courses, restore_course, update_course,
};
use commands::employees::{
    create_employee, get_employee, list_employees, update_employee, delete_employee, get_employee_summary,
};
use commands::groups::{create_group, delete_group, get_group, list_groups, update_group};
use commands::invoices::{
    create_invoice, get_invoice, list_invoices, register_payment, cancel_invoice, get_invoice_summary,
};
use commands::payments::{
    create_payment, delete_payment, get_all_students_payment_summary, get_payment,
    get_student_payment_status, list_payments, list_payments_by_student, update_payment,
};
use commands::payroll::{
    run_payroll, get_payroll_run, list_payroll_runs, get_payroll_summary,
};
use commands::students::{
    create_student, delete_student, get_student, list_students, update_student,
};
use commands::users::{create_user, delete_user, get_user, list_users, update_user};
use infrastructure::database::SqlitePool;
use infrastructure::repositories::{
    SqliteAttendanceRepository, SqliteCourseRepository, SqliteGroupRepository,
    SqlitePaymentRepository, SqliteStudentRepository, SqliteUserRepository,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Get the database path in the app data directory
fn get_db_path() -> PathBuf {
    // Use app data directory for database
    let app_data = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("academix");

    std::fs::create_dir_all(&app_data).ok();
    app_data.join("academix.db")
}

/// Initialize database pool and run migrations
fn init_database() -> SqlitePool {
    let db_path = get_db_path();
    println!("Database path: {:?}", db_path);

    let pool = SqlitePool::new(db_path).expect("Failed to create database pool");

    let conn_ref = pool.connection();
    let conn = conn_ref.lock().unwrap();

    // Run initial schema migration
    let migration_sql = include_str!("../migrations/001_initial_schema.sql");
    conn.execute_batch(migration_sql)
        .expect("Failed to run initial schema migration");

    // Run migration 003 - add guardian and schedule fields
    let migration_003_sql = include_str!("../migrations/003_add_guardian_and_schedule_fields.sql");
    let _ = conn.execute_batch(migration_003_sql);

    // Run migration 004 - add student enrollment columns
    let migration_004_sql = include_str!("../migrations/004_add_student_enrollment_columns.sql");
    let _ = conn.execute_batch(migration_004_sql);

    // Run migration 005 - add course price
    let migration_005_sql = include_str!("../migrations/005_add_course_price.sql");
    let _ = conn.execute_batch(migration_005_sql);

    // Run migration 006 - add group schedule fields
    let migration_006_sql = include_str!("../migrations/006_add_group_schedule_fields.sql");
    let _ = conn.execute_batch(migration_006_sql);

    // Run migration 007 - add start_date column to groups
    let migration_007_sql = include_str!("../migrations/007_add_start_date_to_groups.sql");
    let _ = conn.execute_batch(migration_007_sql);

    // Run migration 008 - fix groups table schema (add end_date, fix columns)
    let migration_008_sql = include_str!("../migrations/008_fix_groups_table_schema.sql");
    let _ = conn.execute_batch(migration_008_sql);

    // Run migration 009 - make payments.due_date nullable
    let migration_009_sql = include_str!("../migrations/009_make_payments_due_date_nullable.sql");
    let _ = conn.execute_batch(migration_009_sql);

    println!("Database initialized successfully");
    drop(conn);
    pool
}

/// Seed admin user if not exists
fn seed_admin_user(pool: &SqlitePool) {
    use crate::application::ports::UserRepository;
    use crate::domain::entities::user::{Role, User};
    use crate::domain::value_objects::Email;

    let repo = SqliteUserRepository::new(Arc::new(pool.clone()));

    // Check if admin exists
    let admin_email = Email::new("admin@academix.com").unwrap();
    if repo.exists_by_email(&admin_email).unwrap_or(false) {
        println!("Admin user already exists");
        return;
    }

    // Create admin user (password: admin123)
    let admin = User::new(
        "admin-1".to_string(),
        "admin@academix.com".to_string(),
        "$2b$12$gghetCr2w7EqfgK5u8jMru4Malw8kQZcXMUQfp2dwOsac2xlo5gYy".to_string(),
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
    GroupService<SqliteGroupRepository>,
    PaymentService<SqlitePaymentRepository, SqliteGroupRepository, SqliteCourseRepository>,
    AttendanceService<SqliteAttendanceRepository>,
) {
    let user_repo = SqliteUserRepository::new(Arc::clone(&pool));
    let student_repo = SqliteStudentRepository::new(Arc::clone(&pool));
    let course_repo = SqliteCourseRepository::new(Arc::clone(&pool));
    let group_repo = SqliteGroupRepository::new(Arc::clone(&pool));
    let payment_repo = SqlitePaymentRepository::new(Arc::clone(&pool));
    let attendance_repo = SqliteAttendanceRepository::new(Arc::clone(&pool));

    (
        UserService::new(user_repo),
        StudentService::new(student_repo, SqliteGroupRepository::new(Arc::clone(&pool))),
        CourseService::new(SqliteCourseRepository::new(Arc::clone(&pool))),
        GroupService::new(group_repo.clone()),
        PaymentService::new(payment_repo, group_repo, course_repo),
        AttendanceService::new(attendance_repo),
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
    ) = create_service_states(Arc::clone(&pool));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AuthAppState::new(Arc::clone(&pool)))
        .manage(user_service)
        .manage(student_service)
        .manage(course_service)
        .manage(group_service)
        .manage(payment_service)
        .manage(attendance_service)
        .invoke_handler(tauri::generate_handler![
            // Health check
            health,
            // Auth commands
            login,
            logout,
            // User commands
            create_user,
            get_user,
            list_users,
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
            // Employee commands
            create_employee,
            get_employee,
            list_employees,
            update_employee,
            delete_employee,
            get_employee_summary,
            // Payroll commands
            run_payroll,
            get_payroll_run,
            list_payroll_runs,
            get_payroll_summary,
            // Accounting commands
            create_entry,
            get_entry,
            list_entries,
            get_trial_balance,
            get_income_statement,
            list_accounts,
            get_account_tree,
            get_accounting_summary,
            // Invoice commands
            create_invoice,
            get_invoice,
            list_invoices,
            register_payment,
            cancel_invoice,
            get_invoice_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
