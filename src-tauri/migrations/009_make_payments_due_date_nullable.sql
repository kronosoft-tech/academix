-- Migration: 009_make_payments_due_date_nullable
-- Description: Make payments.due_date nullable since it's calculated and may not be set
-- Note: SQLite doesn't support ALTER COLUMN, so we use a workaround with a temp table

PRAGMA foreign_keys=OFF;

-- Create backup table with nullable due_date
CREATE TABLE payments_backup AS
SELECT 
    id, student_id, group_id, amount, 
    CASE WHEN due_date = '' THEN NULL ELSE due_date END as due_date,
    paid_date, status, method, reference, description, 
    created_at, updated_at
FROM payments;

DROP TABLE payments;

CREATE TABLE payments (
    id TEXT PRIMARY KEY,
    student_id TEXT NOT NULL,
    group_id TEXT NOT NULL,
    amount REAL NOT NULL,
    due_date TEXT,
    paid_date TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'paid', 'overdue', 'cancelled', 'refunded')),
    method TEXT CHECK(method IN ('cash', 'card', 'transfer', 'online')),
    reference TEXT,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (student_id) REFERENCES students(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups_table(id) ON DELETE CASCADE
);

INSERT INTO payments SELECT * FROM payments_backup;

DROP TABLE payments_backup;

PRAGMA foreign_keys=ON;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_payments_student_id ON payments(student_id);
CREATE INDEX IF NOT EXISTS idx_payments_group_id ON payments(group_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);