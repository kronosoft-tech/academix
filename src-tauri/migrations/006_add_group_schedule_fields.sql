-- Migration: 006_add_group_schedule_fields
-- Description: Add days, start_time, end_time columns to groups_table
-- Date: 2026-03-20

ALTER TABLE groups_table ADD COLUMN days TEXT;
ALTER TABLE groups_table ADD COLUMN start_time TEXT;
ALTER TABLE groups_table ADD COLUMN end_time TEXT;
