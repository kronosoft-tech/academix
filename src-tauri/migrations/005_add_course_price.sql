-- Migration: 005_add_course_price
-- Description: Add price and duration columns to courses
-- Date: 2026-03-19

ALTER TABLE courses ADD COLUMN price REAL DEFAULT 200000;
ALTER TABLE courses ADD COLUMN duration INTEGER DEFAULT 0;