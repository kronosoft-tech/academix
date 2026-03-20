-- Migration: 004_add_student_enrollment_columns
-- Description: Add course_id and group_id to students for direct enrollment tracking
-- Date: 2026-03-19

-- Add enrollment columns to students table
ALTER TABLE students ADD COLUMN course_id TEXT REFERENCES courses(id);
ALTER TABLE students ADD COLUMN group_id TEXT REFERENCES groups_table(id);

-- Create indexes for new columns
CREATE INDEX IF NOT EXISTS idx_students_course_id ON students(course_id);
CREATE INDEX IF NOT EXISTS idx_students_group_id ON students(group_id);
