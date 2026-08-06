//! Control Plane Repository — Turso-backed implementation.
//!
//! Manages user→DB mappings, superadmin users, and sessions
//! in the academix-control-plane Turso database.

use libsql::Database;

/// User-to-database mapping stored in the control plane.
///
/// Maps each user account to their isolated Turso database connection details.
#[derive(Debug, Clone)]
pub struct UserDbMapping {
    pub user_id: String,
    pub email: String,
    pub academy_name: String,
    pub db_url: String,
    pub db_token: String,
    pub org: String,
    pub created_at: String,
}

/// Control plane repository — Turso-backed.
///
/// Manages user→DB mappings, superadmin users, and sessions
/// in the academix-control-plane Turso database.
pub struct ControlPlaneRepository {
    db: Database,
}

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl ControlPlaneRepository {
    /// Connect to control plane Turso DB.
    /// `db_url` and `db_token` come from env vars CONTROL_PLANE_DB_URL / CONTROL_PLANE_DB_TOKEN.
    pub async fn new(db_url: &str, db_token: &str) -> Result<Self, String> {
        let db = libsql::Builder::new_remote(db_url.to_string(), db_token.to_string())
            .build()
            .await
            .map_err(|e| format!("Failed to connect to control plane DB: {}", e))?;
        Ok(Self { db })
    }

    /// Create tables if not exist.
    pub async fn ensure_schema(&self) -> Result<(), String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_databases (
                user_id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                academy_name TEXT NOT NULL,
                db_url TEXT NOT NULL,
                db_token TEXT NOT NULL,
                org TEXT NOT NULL DEFAULT 'academix',
                created_at TEXT NOT NULL
            )",
            (),
        )
        .await
        .map_err(|e| format!("Failed to create user_databases table: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_user_databases_email ON user_databases(email)",
            (),
        )
        .await
        .map_err(|e| format!("Failed to create index: {}", e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'Admin',
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            (),
        )
        .await
        .map_err(|e| format!("Failed to create users table: {}", e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                token TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            (),
        )
        .await
        .map_err(|e| format!("Failed to create sessions table: {}", e))?;
        Ok(())
    }

    /// Save (or replace) a user-to-database mapping.
    pub async fn save_user_db(&self, mapping: &UserDbMapping) -> Result<(), String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO user_databases (user_id, email, academy_name, db_url, db_token, org, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            libsql::params![
                mapping.user_id.clone(),
                mapping.email.clone(),
                mapping.academy_name.clone(),
                mapping.db_url.clone(),
                mapping.db_token.clone(),
                mapping.org.clone(),
                mapping.created_at.clone()
            ],
        )
        .await
        .map_err(|e| format!("Failed to save user DB mapping: {}", e))?;
        Ok(())
    }

    /// Find a user-to-database mapping by email address.
    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserDbMapping>, String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;
        let mut rows = conn
            .query(
                "SELECT user_id, email, academy_name, db_url, db_token, org, created_at FROM user_databases WHERE email = ?1",
                libsql::params![email],
            )
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        match rows
            .next()
            .await
            .map_err(|e| format!("Row fetch error: {}", e))?
        {
            Some(row) => Ok(Some(UserDbMapping {
                user_id: row
                    .get::<String>(0)
                    .map_err(|e| format!("Failed to get user_id: {}", e))?,
                email: row
                    .get::<String>(1)
                    .map_err(|e| format!("Failed to get email: {}", e))?,
                academy_name: row
                    .get::<String>(2)
                    .map_err(|e| format!("Failed to get academy_name: {}", e))?,
                db_url: row
                    .get::<String>(3)
                    .map_err(|e| format!("Failed to get db_url: {}", e))?,
                db_token: row
                    .get::<String>(4)
                    .map_err(|e| format!("Failed to get db_token: {}", e))?,
                org: row
                    .get::<String>(5)
                    .map_err(|e| format!("Failed to get org: {}", e))?,
                created_at: row
                    .get::<String>(6)
                    .map_err(|e| format!("Failed to get created_at: {}", e))?,
            })),
            None => Ok(None),
        }
    }

    /// Find a user-to-database mapping by user ID.
    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Option<UserDbMapping>, String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;
        let mut rows = conn
            .query(
                "SELECT user_id, email, academy_name, db_url, db_token, org, created_at FROM user_databases WHERE user_id = ?1",
                libsql::params![user_id],
            )
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        match rows
            .next()
            .await
            .map_err(|e| format!("Row fetch error: {}", e))?
        {
            Some(row) => Ok(Some(UserDbMapping {
                user_id: row
                    .get::<String>(0)
                    .map_err(|e| format!("Failed to get user_id: {}", e))?,
                email: row
                    .get::<String>(1)
                    .map_err(|e| format!("Failed to get email: {}", e))?,
                academy_name: row
                    .get::<String>(2)
                    .map_err(|e| format!("Failed to get academy_name: {}", e))?,
                db_url: row
                    .get::<String>(3)
                    .map_err(|e| format!("Failed to get db_url: {}", e))?,
                db_token: row
                    .get::<String>(4)
                    .map_err(|e| format!("Failed to get db_token: {}", e))?,
                org: row
                    .get::<String>(5)
                    .map_err(|e| format!("Failed to get org: {}", e))?,
                created_at: row
                    .get::<String>(6)
                    .map_err(|e| format!("Failed to get created_at: {}", e))?,
            })),
            None => Ok(None),
        }
    }

    /// List all user-to-database mappings (for superadmin).
    pub async fn list_all_databases(&self) -> Result<Vec<UserDbMapping>, String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;
        let mut rows = conn
            .query(
                "SELECT user_id, email, academy_name, db_url, db_token, org, created_at FROM user_databases ORDER BY created_at DESC",
                (),
            )
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| format!("Row fetch error: {}", e))?
        {
            results.push(UserDbMapping {
                user_id: row
                    .get::<String>(0)
                    .map_err(|e| format!("Failed to get user_id: {}", e))?,
                email: row
                    .get::<String>(1)
                    .map_err(|e| format!("Failed to get email: {}", e))?,
                academy_name: row
                    .get::<String>(2)
                    .map_err(|e| format!("Failed to get academy_name: {}", e))?,
                db_url: row
                    .get::<String>(3)
                    .map_err(|e| format!("Failed to get db_url: {}", e))?,
                db_token: row
                    .get::<String>(4)
                    .map_err(|e| format!("Failed to get db_token: {}", e))?,
                org: row
                    .get::<String>(5)
                    .map_err(|e| format!("Failed to get org: {}", e))?,
                created_at: row
                    .get::<String>(6)
                    .map_err(|e| format!("Failed to get created_at: {}", e))?,
            });
        }
        Ok(results)
    }

    /// Save (or replace) a user in the control plane.
    pub async fn save_user(&self, user: &UserRow) -> Result<(), String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;
        conn.execute(
            "INSERT OR REPLACE INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            libsql::params![
                user.id.clone(),
                user.email.clone(),
                user.password_hash.clone(),
                user.name.clone(),
                user.role.clone(),
                user.is_active as i32,
                user.created_at.clone(),
                user.updated_at.clone()
            ],
        )
        .await
        .map_err(|e| format!("Failed to save user: {}", e))?;
        Ok(())
    }

    /// Get the subscription status for a user from the control plane DB.
    /// Returns Some((status, plan)) or None if no subscription exists.
    pub async fn get_subscription_status(
        &self,
        user_id: &str,
    ) -> Result<Option<(String, Option<String>)>, String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;
        let mut rows = conn
            .query(
                "SELECT status, plan FROM subscriptions WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 1",
                libsql::params![user_id],
            )
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        match rows
            .next()
            .await
            .map_err(|e| format!("Row fetch error: {}", e))?
        {
            Some(row) => {
                let status: String = row
                    .get::<String>(0)
                    .map_err(|e| format!("Failed to get status: {}", e))?;
                let plan: Option<String> = row.get::<String>(1).ok();
                Ok(Some((status, plan)))
            }
            None => Ok(None),
        }
    }

    /// Find a control-plane user by email address.
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRow>, String> {
        let conn = self
            .db
            .connect()
            .map_err(|e| format!("Connection error: {}", e))?;
        let mut rows = conn
            .query(
                "SELECT id, email, password_hash, name, role, is_active, created_at, updated_at FROM users WHERE email = ?1",
                libsql::params![email],
            )
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        match rows
            .next()
            .await
            .map_err(|e| format!("Row fetch error: {}", e))?
        {
            Some(row) => Ok(Some(UserRow {
                id: row
                    .get::<String>(0)
                    .map_err(|e| format!("Failed to get id: {}", e))?,
                email: row
                    .get::<String>(1)
                    .map_err(|e| format!("Failed to get email: {}", e))?,
                password_hash: row
                    .get::<String>(2)
                    .map_err(|e| format!("Failed to get password_hash: {}", e))?,
                name: row
                    .get::<String>(3)
                    .map_err(|e| format!("Failed to get name: {}", e))?,
                role: row
                    .get::<String>(4)
                    .map_err(|e| format!("Failed to get role: {}", e))?,
                is_active: {
                    let val: i32 = row
                        .get::<i32>(5)
                        .map_err(|e| format!("Failed to get is_active: {}", e))?;
                    val != 0
                },
                created_at: row
                    .get::<String>(6)
                    .map_err(|e| format!("Failed to get created_at: {}", e))?,
                updated_at: row
                    .get::<String>(7)
                    .map_err(|e| format!("Failed to get updated_at: {}", e))?,
            })),
            None => Ok(None),
        }
    }
}
