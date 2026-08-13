-- Migration 017: Add app_settings table for configurable settings
-- Stores key-value pairs for application settings like attendance threshold

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Insert default attendance threshold (3 absences triggers warning)
INSERT OR IGNORE INTO app_settings (key, value, updated_at)
VALUES ('attendance_threshold', '3', datetime('now'));