-- Migration 018: Add class_duration and skipped_dates to groups
-- class_duration: duration of each class session in minutes
-- skipped_dates: JSON array of dates to skip (e.g., holidays)

-- Add class_duration column (nullable for backward compatibility)
ALTER TABLE groups_table ADD COLUMN class_duration INTEGER DEFAULT NULL;

-- Add skipped_dates column (nullable, defaults to empty JSON array)
ALTER TABLE groups_table ADD COLUMN skipped_dates TEXT DEFAULT '[]';
