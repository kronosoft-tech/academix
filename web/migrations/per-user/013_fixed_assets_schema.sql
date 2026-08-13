-- Migration: 013_fixed_assets_schema
-- Description: Fixed assets (activos fijos) for balance - machinery, equipment, vehicles
-- Created: 2026-04-23
--
-- Fixed assets are long-term assets used in operations, not for sale
-- Depreciation is calculated separately

CREATE TABLE IF NOT EXISTS fixed_assets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL CHECK(asset_type IN ('equipment', 'furniture', 'vehicle', 'computer', 'building', 'land', 'other')),
    description TEXT,
    acquisition_date TEXT NOT NULL,
    acquisition_cost REAL NOT NULL,
    current_value REAL NOT NULL,
    useful_life_years INTEGER DEFAULT 5,
    depreciation_method TEXT DEFAULT 'straight_line',
    account_code TEXT,  -- Link to chart of accounts (15xx - Activos Fijos)
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'depreciated', 'disposed', 'sold')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fixed_assets_status ON fixed_assets(status);
CREATE INDEX IF NOT EXISTS idx_fixed_assets_type ON fixed_assets(asset_type);

-- Table to track depreciation entries
CREATE TABLE IF NOT EXISTS asset_depreciation (
    id TEXT PRIMARY KEY,
    fixed_asset_id TEXT NOT NULL,
    period_year INTEGER NOT NULL,
    period_month INTEGER NOT NULL,
    depreciation_amount REAL NOT NULL,
    accumulated_depreciation REAL NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (fixed_asset_id) REFERENCES fixed_assets(id)
);

CREATE INDEX IF NOT EXISTS idx_asset_depreciation_asset ON asset_depreciation(fixed_asset_id);