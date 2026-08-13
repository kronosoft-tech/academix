-- Migration: 007_add_start_date_to_groups
-- Description: Add start_date column to groups_table to track group start dates
-- Updated: 2026-03-26

-- Add start_date column to groups_table
ALTER TABLE groups_table ADD COLUMN start_date TEXT;

-- Create index for faster queries on start_date
CREATE INDEX IF NOT EXISTS idx_groups_start_date ON groups_table(start_date);