-- Migration: 014_fixed_assets_accounts
-- Description: Add missing fixed asset accounts (15xx and 16xx)
-- Created: 2026-04-23
--
-- 15 - Intangibles
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('15', '15', 'INTANGIBLES', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1505', '1505', 'Plusvalía (Goodwill)', 'asset', '15', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1510', '1510', 'Marcas', 'asset', '15', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1515', '1515', 'Patentes y Derechos', 'asset', '15', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1520', '1520', 'Licencias y Software', 'asset', '15', 0, 1, datetime('now'), datetime('now'));

-- 16 - Propiedad, Planta y Equipo (already exists, adding more accounts)
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1605', '1605', 'Terrenos', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1610', '1610', 'Edificios', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1615', '1615', 'Muebles y Enseres', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1620', '1620', 'Equipo de Oficina', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1625', '1625', 'Equipos de Computación', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1630', '1630', 'Equipos de Comunicación', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1635', '1635', 'Maquinaria y Equipo', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1640', '1640', 'Vehículos', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1645', '1645', 'Herramientas', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1650', '1650', 'Equipos de Sonido', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1655', '1655', 'Equipos de Video', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1660', '1660', 'Equipos de Aire Acondicionado', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1665', '1665', 'Muebles de Cocina', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1670', '1670', 'Equipos de Seguridad', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1675', '1675', 'Equipos de Gimnasio', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1680', '1680', 'Instrumentos Musicales', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1685', '1685', 'Equipos de Laboratorio', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1690', '1690', 'Otros Activos Fijos', 'asset', '16', 0, 1, datetime('now'), datetime('now'));

-- Depreciación Acumulada
INSERT OR IGNORE INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1699', '1699', 'Depreciación Acumulada', 'asset', '16', 0, 1, datetime('now'), datetime('now'));