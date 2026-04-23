-- Migration: 015_pasivos_accounts
-- Add missing pasivo/liability accounts (21xx and 22xx)
-- Created: 2026-04-23

-- 21 - Proveedores (already exists)
-- Adding more proveedor accounts
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2110', '2110', 'Proveedores del Exterior', 'liability', '21', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2120', '2120', 'Cuentas por Pagar', 'liability', '21', 0, 1, datetime('now'), datetime('now'));

-- 22 - Obligaciones Laborales
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('22', '22', 'OBLIGACIONES LABORALES', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2205', '2205', 'Salarios por Pagar', 'liability', '22', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2210', '2210', 'Cesantías Consignadas', 'liability', '22', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2215', '2215', 'Intereses sobre Cesantías', 'liability', '22', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2220', '2220', 'Primas por Pagar', 'liability', '22', 0, 1, datetime('now'), datetime('now'));

-- 23 - Impuestos
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('23', '23', 'IMPUESTOS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2305', '2305', 'IVA por Pagar', 'liability', '23', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2310', '2310', 'Retención en la Fuente', 'liability', '23', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2315', '2315', 'Impuesto de Renta', 'liability', '23', 0, 1, datetime('now'), datetime('now'));

-- 24 - Prestamos
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('24', '24', 'PRÉSTAMOS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2405', '2405', 'Prestamo Bancario CP', 'liability', '24', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2410', '2410', 'Prestamo Bancario LP', 'liability', '24', 0, 1, datetime('now'), datetime('now'));

-- 28 - Provisiones (already exists partially)
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2810', '2810', 'Provisión Obligaciones', 'liability', '28', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2815', '2815', 'Provisión Litigios', 'liability', '28', 0, 1, datetime('now'), datetime('now'));