-- Migration: 003_add_guardian_and_schedule_fields
-- Description: Add guardian information fields to students and schedule field to groups
-- Date: 2026-03-19

-- Add guardian and email fields to students table
ALTER TABLE students ADD COLUMN email TEXT NOT NULL DEFAULT '';
ALTER TABLE students ADD COLUMN guardian_name TEXT;
ALTER TABLE students ADD COLUMN guardian_document TEXT;
ALTER TABLE students ADD COLUMN guardian_phone TEXT;

-- Add schedule field to groups_table
ALTER TABLE groups_table ADD COLUMN schedule TEXT;

-- Create indexes for new fields
CREATE INDEX IF NOT EXISTS idx_students_guardian_name ON students(guardian_name);
CREATE INDEX IF NOT EXISTS idx_groups_schedule ON groups_table(schedule);