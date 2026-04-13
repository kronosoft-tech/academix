/**
 * Database Initialization Module
 * 
 * Handles database migrations and initial setup for Academix MVP.
 * Uses tauri-plugin-sql for SQLite operations.
 */

import Database from '@tauri-apps/plugin-sql';

export interface Migration {
  version: number;
  name: string;
  sql: string;
}

/**
 * Get database migrations
 * These are embedded SQL migration scripts
 */
export function getMigrations(): Migration[] {
  return [
    {
      version: 1,
      name: 'initial_schema',
      sql: `
-- ============================================
-- USERS - Admin, Gerente, Empleado, Profesor
-- ============================================
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('Admin', 'Gerente', 'Empleado', 'Profesor')),
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- ============================================
-- SESSIONS - User login sessions
-- ============================================
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);

-- ============================================
-- STUDENTS - Alumnos
-- ============================================
CREATE TABLE IF NOT EXISTS students (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    address TEXT,
    birth_date TEXT,
    guardian_name TEXT,
    guardian_phone TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'inactive', 'graduated', 'suspended')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_students_status ON students(status);
CREATE INDEX IF NOT EXISTS idx_students_name ON students(name);

-- ============================================
-- COURSES - Cursos
-- ============================================
CREATE TABLE IF NOT EXISTS courses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'inactive', 'archived')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_courses_status ON courses(status);
CREATE INDEX IF NOT EXISTS idx_courses_category ON courses(category);

-- ============================================
-- GROUPS - Grupos
-- ============================================
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    schedule TEXT,
    max_students INTEGER NOT NULL DEFAULT 20,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'inactive', 'completed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_groups_course_id ON groups(course_id);
CREATE INDEX IF NOT EXISTS idx_groups_status ON groups(status);

-- ============================================
-- GROUP_STUDENTS - Relación grupos-alumnos
-- ============================================
CREATE TABLE IF NOT EXISTS group_students (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    student_id INTEGER NOT NULL,
    enrolled_at TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'inactive', 'completed', 'dropped')),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE,
    UNIQUE(group_id, student_id)
);

CREATE INDEX IF NOT EXISTS idx_group_students_group_id ON group_students(group_id);
CREATE INDEX IF NOT EXISTS idx_group_students_student_id ON group_students(student_id);

-- ============================================
-- SCHEDULES - Horarios
-- ============================================
CREATE TABLE IF NOT EXISTS schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 0 AND 6),
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    room TEXT,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_schedules_group_id ON schedules(group_id);

-- ============================================
-- PAYMENTS - Pagos
-- ============================================
CREATE TABLE IF NOT EXISTS payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    amount REAL NOT NULL,
    due_date TEXT NOT NULL,
    paid_date TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'paid', 'overdue', 'cancelled', 'refunded')),
    method TEXT CHECK(method IN ('cash', 'card', 'transfer', 'online')),
    reference TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payments_student_id ON payments(student_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX IF NOT EXISTS idx_payments_due_date ON payments(due_date);

-- ============================================
-- ATTENDANCE - Asistencias
-- ============================================
CREATE TABLE IF NOT EXISTS attendance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('present', 'absent', 'late', 'excused')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    UNIQUE(student_id, group_id, date)
);

CREATE INDEX IF NOT EXISTS idx_attendance_student_id ON attendance(student_id);
CREATE INDEX IF NOT EXISTS idx_attendance_group_id ON attendance(group_id);
CREATE INDEX IF NOT EXISTS idx_attendance_date ON attendance(date);

-- ============================================
-- EVALUATIONS - Evaluaciones
-- ============================================
CREATE TABLE IF NOT EXISTS evaluations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0,
    due_date TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_evaluations_group_id ON evaluations(group_id);

-- ============================================
-- GRADES - Calificaciones
-- ============================================
CREATE TABLE IF NOT EXISTS grades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL,
    evaluation_id INTEGER NOT NULL,
    grade REAL NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE,
    FOREIGN KEY (evaluation_id) REFERENCES evaluations(id) ON DELETE CASCADE,
    UNIQUE(student_id, evaluation_id)
);

CREATE INDEX IF NOT EXISTS idx_grades_student_id ON grades(student_id);
CREATE INDEX IF NOT EXISTS idx_grades_evaluation_id ON grades(evaluation_id);

-- ============================================
-- PERMISSIONS - Permisos configurables
-- ============================================
CREATE TABLE IF NOT EXISTS permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role TEXT NOT NULL,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    allowed INTEGER NOT NULL DEFAULT 1,
    UNIQUE(role, resource, action)
);

CREATE INDEX IF NOT EXISTS idx_permissions_role ON permissions(role);

-- ============================================
-- CONFIGURATION - Configuración de la academia
-- ============================================
CREATE TABLE IF NOT EXISTS configuration (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,
    value TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default configuration values
INSERT OR IGNORE INTO configuration (key, value) VALUES 
    ('academy_name', 'Academix'),
    ('academy_address', ''),
    ('academy_phone', ''),
    ('academy_email', ''),
    ('currency', 'COP'),
    ('timezone', 'America/Bogota');

-- Insert default permissions
INSERT OR IGNORE INTO permissions (role, resource, action, allowed) VALUES
-- Admin has full access
('Admin', 'users', 'create', 1),
('Admin', 'users', 'read', 1),
('Admin', 'users', 'update', 1),
('Admin', 'users', 'delete', 1),
('Admin', 'students', 'create', 1),
('Admin', 'students', 'read', 1),
('Admin', 'students', 'update', 1),
('Admin', 'students', 'delete', 1),
('Admin', 'courses', 'create', 1),
('Admin', 'courses', 'read', 1),
('Admin', 'courses', 'update', 1),
('Admin', 'courses', 'delete', 1),
('Admin', 'groups', 'create', 1),
('Admin', 'groups', 'read', 1),
('Admin', 'groups', 'update', 1),
('Admin', 'groups', 'delete', 1),
('Admin', 'payments', 'create', 1),
('Admin', 'payments', 'read', 1),
('Admin', 'payments', 'update', 1),
('Admin', 'payments', 'delete', 1),
('Admin', 'attendance', 'create', 1),
('Admin', 'attendance', 'read', 1),
('Admin', 'attendance', 'update', 1),
('Admin', 'attendance', 'delete', 1),
('Admin', 'evaluations', 'create', 1),
('Admin', 'evaluations', 'read', 1),
('Admin', 'evaluations', 'update', 1),
('Admin', 'evaluations', 'delete', 1),
('Admin', 'grades', 'create', 1),
('Admin', 'grades', 'read', 1),
('Admin', 'grades', 'update', 1),
('Admin', 'grades', 'delete', 1),
('Admin', 'configuration', 'read', 1),
('Admin', 'configuration', 'update', 1),
-- Gerente has most access but cannot delete users
('Gerente', 'users', 'create', 1),
('Gerente', 'users', 'read', 1),
('Gerente', 'users', 'update', 1),
('Gerente', 'users', 'delete', 0),
('Gerente', 'students', 'create', 1),
('Gerente', 'students', 'read', 1),
('Gerente', 'students', 'update', 1),
('Gerente', 'students', 'delete', 1),
('Gerente', 'courses', 'create', 1),
('Gerente', 'courses', 'read', 1),
('Gerente', 'courses', 'update', 1),
('Gerente', 'courses', 'delete', 1),
('Gerente', 'groups', 'create', 1),
('Gerente', 'groups', 'read', 1),
('Gerente', 'groups', 'update', 1),
('Gerente', 'groups', 'delete', 1),
('Gerente', 'payments', 'create', 1),
('Gerente', 'payments', 'read', 1),
('Gerente', 'payments', 'update', 1),
('Gerente', 'payments', 'delete', 1),
('Gerente', 'attendance', 'create', 1),
('Gerente', 'attendance', 'read', 1),
('Gerente', 'attendance', 'update', 1),
('Gerente', 'attendance', 'delete', 1),
('Gerente', 'evaluations', 'create', 1),
('Gerente', 'evaluations', 'read', 1),
('Gerente', 'evaluations', 'update', 1),
('Gerente', 'evaluations', 'delete', 1),
('Gerente', 'grades', 'create', 1),
('Gerente', 'grades', 'read', 1),
('Gerente', 'grades', 'update', 1),
('Gerente', 'grades', 'delete', 1),
('Gerente', 'configuration', 'read', 1),
('Gerente', 'configuration', 'update', 0),
-- Empleado can manage students and attendance
('Empleado', 'users', 'read', 0),
('Empleado', 'students', 'create', 1),
('Empleado', 'students', 'read', 1),
('Empleado', 'students', 'update', 1),
('Empleado', 'students', 'delete', 0),
('Empleado', 'courses', 'read', 1),
('Empleado', 'courses', 'create', 0),
('Empleado', 'courses', 'update', 0),
('Empleado', 'courses', 'delete', 0),
('Empleado', 'groups', 'read', 1),
('Empleado', 'groups', 'create', 0),
('Empleado', 'groups', 'update', 0),
('Empleado', 'groups', 'delete', 0),
('Empleado', 'payments', 'create', 1),
('Empleado', 'payments', 'read', 1),
('Empleado', 'payments', 'update', 1),
('Empleado', 'payments', 'delete', 0),
('Empleado', 'attendance', 'create', 1),
('Empleado', 'attendance', 'read', 1),
('Empleado', 'attendance', 'update', 1),
('Empleado', 'attendance', 'delete', 0),
('Empleado', 'evaluations', 'read', 1),
('Empleado', 'evaluations', 'create', 0),
('Empleado', 'evaluations', 'update', 0),
('Empleado', 'evaluations', 'delete', 0),
('Empleado', 'grades', 'read', 1),
('Empleado', 'grades', 'create', 0),
('Empleado', 'grades', 'update', 0),
('Empleado', 'grades', 'delete', 0),
-- Profesor can only manage attendance and grades for their groups
('Profesor', 'users', 'read', 0),
('Profesor', 'students', 'read', 1),
('Profesor', 'students', 'create', 0),
('Profesor', 'students', 'update', 0),
('Profesor', 'students', 'delete', 0),
('Profesor', 'courses', 'read', 1),
('Profesor', 'courses', 'create', 0),
('Profesor', 'courses', 'update', 0),
('Profesor', 'courses', 'delete', 0),
('Profesor', 'groups', 'read', 1),
('Profesor', 'groups', 'create', 0),
('Profesor', 'groups', 'update', 0),
('Profesor', 'groups', 'delete', 0),
('Profesor', 'payments', 'read', 0),
('Profesor', 'payments', 'create', 0),
('Profesor', 'payments', 'update', 0),
('Profesor', 'payments', 'delete', 0),
('Profesor', 'attendance', 'create', 1),
('Profesor', 'attendance', 'read', 1),
('Profesor', 'attendance', 'update', 1),
('Profesor', 'attendance', 'delete', 0),
('Profesor', 'evaluations', 'create', 1),
('Profesor', 'evaluations', 'read', 1),
('Profesor', 'evaluations', 'update', 1),
('Profesor', 'evaluations', 'delete', 0),
('Profesor', 'grades', 'create', 1),
('Profesor', 'grades', 'read', 1),
('Profesor', 'grades', 'update', 1),
('Profesor', 'grades', 'delete', 0);
      `
    }
  ];
}

/**
 * Database singleton instance
 */
let db: Database | null = null;

/**
 * Initialize the database with migrations
 */
export async function initDatabase(): Promise<Database> {
  if (db) {
    return db;
  }

  db = await Database.load('sqlite:academix.db');
  
  const migrations = getMigrations();
  
  // Run migrations
  for (const migration of migrations) {
    // Split SQL into individual statements
    const statements = migration.sql
      .split(';')
      .map(s => s.trim())
      .filter(s => s.length > 0 && !s.startsWith('--'));
    
    for (const statement of statements) {
      try {
        await db.execute(statement);
      } catch (error) {
        // Ignore "table already exists" errors
        const errorStr = String(error);
        if (!errorStr.includes('already exists')) {
          console.error(`Migration ${migration.version} (${migration.name}) failed:`, error);
          throw error;
        }
      }
    }
  }
  
  console.log(`Database initialized with ${migrations.length} migrations`);
  return db;
}

/**
 * Get the database instance
 */
export function getDatabase(): Database {
  if (!db) {
    throw new Error('Database not initialized. Call initDatabase() first.');
  }
  return db;
}

/**
 * Close the database connection
 */
export async function closeDatabase(): Promise<void> {
  if (db) {
    await db.close();
    db = null;
  }
}
