-- Migration: 008_fix_groups_table_schema
-- Description: Fix groups_table schema to include end_date column
-- Updated: 2026-03-26

-- Add end_date column to groups_table
ALTER TABLE groups_table ADD COLUMN end_date TEXT;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_groups_start_date ON groups_table(start_date);
CREATE INDEX IF NOT EXISTS idx_groups_end_date ON groups_table(end_date);