-- Migration: 008_fix_groups_table_schema
-- Description: Fix groups_table schema to include proper start_date and end_date columns (nullable)
-- Updated: 2026-03-26

BEGIN TRANSACTION;

-- Create temporary table with backup of existing data
CREATE TABLE groups_table_backup AS
SELECT 
    id,
    course_id,
    name,
    COALESCE(professor_id, '') as professor_id,
    COALESCE(schedule, '') as schedule,
    COALESCE(days, '[]') as days,
    COALESCE(start_time, '') as start_time,
    COALESCE(end_time, '') as end_time,
    -- For start_date: try to get it if column exists, otherwise use created_at
    CASE 
        WHEN (SELECT count(*) FROM pragma_table_info('groups_table') WHERE name='start_date') > 0 
        THEN start_date 
        ELSE created_at 
    END as start_date,
    -- end_date: new column, set to empty string for now (we'll treat empty as NULL)
    '' as end_date,
    max_students,
    current_students,
    COALESCE(status, 'open') as status,
    COALESCE(created_at, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) as created_at,
    COALESCE(updated_at, strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) as updated_at
FROM groups_table;

-- Drop original table
DROP TABLE groups_table;

-- Create new table with correct schema
CREATE TABLE groups_table (
    id TEXT NOT NULL PRIMARY KEY,
    course_id TEXT NOT NULL,
    name TEXT NOT NULL,
    professor_id TEXT NULL,
    schedule TEXT NULL,
    days TEXT NULL,
    start_time TEXT NULL,
    end_time TEXT NULL,
    start_date TEXT NULL,
    end_date TEXT NULL,
    max_students INTEGER NOT NULL,
    current_students INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Copy data from backup to new table
INSERT INTO groups_table (
    id, course_id, name, professor_id, schedule, days, 
    start_time, end_time, start_date, end_date, 
    max_students, current_students, status, created_at, updated_at
)
SELECT 
    id, course_id, name, 
    NULLIF(professor_id, ''), 
    NULLIF(schedule, ''), 
    NULLIF(days, '[]'), 
    NULLIF(start_time, ''), 
    NULLIF(end_time, ''), 
    NULLIF(start_date, ''), 
    NULLIF(end_date, ''), 
    max_students, 
    current_students, 
    status, 
    created_at, 
    updated_at
FROM groups_table_backup;

-- Drop backup table
DROP TABLE groups_table_backup;

COMMIT;

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_groups_start_date ON groups_table(start_date);
CREATE INDEX IF NOT EXISTS idx_groups_end_date ON groups_table(end_date);