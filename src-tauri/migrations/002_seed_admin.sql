-- Migration: 002_seed_admin_user
-- Description: Seed admin user for initial access
-- Created: 2026-03-18

-- Insert admin user (password: admin123)
INSERT OR IGNORE INTO users (email, password_hash, name, role, is_active) 
VALUES ('admin@academix.com', '$2b$12$gghetCr2w7EqfgK5u8jMru4Malw8kQZcXMUQfp2dwOsac2xlo5gYy', 'Luifer Admin', 'Admin', 1);
