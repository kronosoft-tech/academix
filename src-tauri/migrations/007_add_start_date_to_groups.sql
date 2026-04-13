-- Migration: 007_add_start_date_to_groups
-- Description: Add start_date column to groups_table to track group start dates
-- Updated: 2026-03-26
-- 
-- This migration adds the start_date column to groups_table to track when each group started.
-- The column is NOT NULL with CURRENT_TIMESTAMP as default for existing records.

-- ============================================
-- Add start_date column to groups_table
-- ============================================
ALTER TABLE groups_table ADD COLUMN start_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- Create index for faster queries on start_date
CREATE INDEX IF NOT EXISTS idx_groups_start_date ON groups_table(start_date);

-- ============================================
-- Migration verification
-- ============================================
-- Verify the column was added successfully
-- SELECT sql FROM sqlite_master WHERE type='table' AND name='groups_table';