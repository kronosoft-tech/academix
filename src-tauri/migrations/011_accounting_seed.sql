-- Migration: 011_accounting_seed
-- Description: Seed standard Colombian chart of accounts (PUC)
-- Created: 2026-04-13
-- 
-- Plan Único de Cuentas Colombiano (PUC)
-- Estructura: X - X.X - X.X.X
-- 1xxx: Activos
-- 2xxx: Pasivos
-- 3xxx: Patrimonio
-- 4xxx: Gastos
-- 5xxx: Costos de Producción
-- 6xxx: Ingresos

-- ============================================
-- 1xxx - ACTIVOS
-- ============================================
-- 11 - Caja y Bancos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1105', 'CAJA', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '110501', 'Caja General', 'asset', (SELECT id FROM account_categories WHERE code = '1105'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '110502', 'Caja Menor', 'asset', (SELECT id FROM account_categories WHERE code = '1105'), 0, 1, datetime('now'), datetime('now'));

INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1110', 'BANCOS', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '111001', 'Banco de Colombia', 'asset', (SELECT id FROM account_categories WHERE code = '1110'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '111002', 'Banco Davivienda', 'asset', (SELECT id FROM account_categories WHERE code = '1110'), 0, 1, datetime('now'), datetime('now'));

-- 12 - Inversiones Temporales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '12', 'INVERSIONES TEMPORALES', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));

-- 13 - Cuentas por Cobrar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1305', 'CLIENTES', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '130501', 'Cuentas por Cobrar - Matrículas', 'asset', (SELECT id FROM account_categories WHERE code = '1305'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '130502', 'Cuentas por Cobrar - Mensualidades', 'asset', (SELECT id FROM account_categories WHERE code = '1305'), 0, 1, datetime('now'), datetime('now'));

INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1380', 'OTRAS CUENTAS POR COBRAR', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));

-- 14 - Otras Cuentas por Cobrar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '14', 'OTRAS CUENTAS POR COBRAR', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));

-- 15 - Inventarios
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '15', 'INVENTARIOS', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1516', 'Materiales y Suministros', 'asset', (SELECT id FROM account_categories WHERE code = '15'), 0, 1, datetime('now'), datetime('now'));

-- 16 - Propiedad, Planta y Equipo
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '16', 'PROPIEDAD, PLANTA Y EQUIPO', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1615', 'Muebles y Enseres', 'asset', (SELECT id FROM account_categories WHERE code = '16'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1616', 'Equipos de Oficina', 'asset', (SELECT id FROM account_categories WHERE code = '16'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1617', 'Equipos de Computación', 'asset', (SELECT id FROM account_categories WHERE code = '16'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '1620', 'Depreciación Acumulada', 'asset', (SELECT id FROM account_categories WHERE code = '16'), 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 2xxx - PASIVOS
-- ============================================
-- 21 - Proveedores
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '21', 'PROVEEDORES', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2105', 'Proveedores Nacionales', 'liability', (SELECT id FROM account_categories WHERE code = '21'), 0, 1, datetime('now'), datetime('now'));

-- 22 - Cuentas por Pagar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '22', 'CUENTAS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));

-- 23 - Obligaciones Laborales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '23', 'OBLIGACIONES LABORALES', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2310', 'Salarios por Pagar', 'liability', (SELECT id FROM account_categories WHERE code = '23'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2315', 'Cesantías Consignadas', 'liability', (SELECT id FROM account_categories WHERE code = '23'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2320', 'Intereses sobre Cesantías', 'liability', (SELECT id FROM account_categories WHERE code = '23'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2330', 'Prima de Servicios', 'liability', (SELECT id FROM account_categories WHERE code = '23'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2340', 'Vacaciones', 'liability', (SELECT id FROM account_categories WHERE code = '23'), 0, 1, datetime('now'), datetime('now'));

-- 24 - Impuestos por Pagar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '24', 'IMPUESTOS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2408', 'IVA por Pagar', 'liability', (SELECT id FROM account_categories WHERE code = '24'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2412', 'Retención en la Fuente', 'liability', (SELECT id FROM account_categories WHERE code = '24'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '2436', 'ReteIVA', 'liability', (SELECT id FROM account_categories WHERE code = '24'), 0, 1, datetime('now'), datetime('now'));

-- 25 - Obligaciones Financieras
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '25', 'OBLIGACIONES FINANCIERAS', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));

-- 26 - Otras Cuentas por Pagar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '26', 'OTRAS CUENTAS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 3xxx - PATRIMONIO
-- ============================================
-- 31 - Capital
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '31', 'CAPITAL', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '3115', 'Capital Suscrito y Pagado', 'equity', (SELECT id FROM account_categories WHERE code = '31'), 0, 1, datetime('now'), datetime('now'));

-- 32 - Reservas
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '32', 'RESERVAS', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '3210', 'Reserva Legal', 'equity', (SELECT id FROM account_categories WHERE code = '32'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '3220', 'Reservas Ocasionales', 'equity', (SELECT id FROM account_categories WHERE code = '32'), 0, 1, datetime('now'), datetime('now'));

-- 33 - Resultados del Ejercicio
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '33', 'RESULTADOS DEL EJERCICIO', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '3310', 'Utilidades del Ejercicio', 'equity', (SELECT id FROM account_categories WHERE code = '33'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '3320', 'Pérdidas del Ejercicio', 'equity', (SELECT id FROM account_categories WHERE code = '33'), 0, 1, datetime('now'), datetime('now'));

-- 34 - Resultados Acumulados
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '34', 'RESULTADOS ACUMULADOS', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '3410', 'Utilidades Acumuladas', 'equity', (SELECT id FROM account_categories WHERE code = '34'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '3420', 'Pérdidas Acumuladas', 'equity', (SELECT id FROM account_categories WHERE code = '34'), 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 4xxx - GASTOS
-- ============================================
-- 41 - Gastos de Personal
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '41', 'GASTOS DE PERSONAL', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4105', 'Salarios', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4110', 'Auxilio de Transporte', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4115', 'Horas Extras', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4120', 'Comisiones', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4130', 'Cesantías', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4135', 'Intereses sobre Cesantías', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4140', 'Prima de Servicios', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4145', 'Vacaciones', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4150', 'Aporte a Salud (EPS)', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4155', 'Aporte a Pensión (AFP)', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4160', 'Aporte a Riesgos Laborales (ARL)', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4165', 'Aporte a ICBF y SENA', 'expense', (SELECT id FROM account_categories WHERE code = '41'), 0, 1, datetime('now'), datetime('now'));

-- 42 - Gastos de Operación
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '42', 'GASTOS DE OPERACIÓN', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4210', 'Arrendamientos', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4215', 'Servicios', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4220', 'Gas', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4225', 'Energía', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4230', 'Agua y Alcantarillado', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4235', 'Teléfono e Internet', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4240', 'Mantenimiento y Reparaciones', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4245', 'Gastos de Representación', 'expense', (SELECT id FROM account_categories WHERE code = '42'), 0, 1, datetime('now'), datetime('now'));

-- 43 - Depreciaciones
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '43', 'DEPRECIACIONES', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));

-- 44 - Gastos Financieros
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '44', 'GASTOS FINANCIEROS', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4410', 'Intereses', 'expense', (SELECT id FROM account_categories WHERE code = '44'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4420', 'Comisiones Bancarias', 'expense', (SELECT id FROM account_categories WHERE code = '44'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4430', 'GMF (4x1000)', 'expense', (SELECT id FROM account_categories WHERE code = '44'), 0, 1, datetime('now'), datetime('now'));

-- 45 - Otros Gastos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '45', 'OTROS GASTOS', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4510', 'Gastos Extraordinarios', 'expense', (SELECT id FROM account_categories WHERE code = '45'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4520', 'Gastos Varios', 'expense', (SELECT id FROM account_categories WHERE code = '45'), 0, 1, datetime('now'), datetime('now'));

-- 48 - Gastos por Impuestos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '48', 'GASTOS POR IMPUESTOS', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4810', 'Impuesto de Industria y Comercio (ICA)', 'expense', (SELECT id FROM account_categories WHERE code = '48'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '4815', 'Gravamen a los Movimientos Financieros', 'expense', (SELECT id FROM account_categories WHERE code = '48'), 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 5xxx - COSTOS DE PRODUCCIÓN
-- ============================================
-- 51 - Costos de Operación
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '51', 'COSTOS DE OPERACIÓN', 'cost', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '5105', 'Costos de Educación', 'cost', (SELECT id FROM account_categories WHERE code = '51'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '5110', 'Materiales Didácticos', 'cost', (SELECT id FROM account_categories WHERE code = '51'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '5115', 'Gastos de Personal Docente', 'cost', (SELECT id FROM account_categories WHERE code = '51'), 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 6xxx - INGRESOS
-- ============================================
-- 61 - Ingresos Operacionales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '61', 'INGRESOS OPERACIONALES', 'income', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6105', 'SERVICIOS EDUCATIVOS', 'income', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6110', 'Matrículas', 'income', (SELECT id FROM account_categories WHERE code = '6105'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6115', 'Mensualidades', 'income', (SELECT id FROM account_categories WHERE code = '6105'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6120', 'Cursos Especiales', 'income', (SELECT id FROM account_categories WHERE code = '6105'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6125', 'Certificaciones', 'income', (SELECT id FROM account_categories WHERE code = '6105'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6130', 'Uniformes y Materiales', 'income', (SELECT id FROM account_categories WHERE code = '6105'), 0, 1, datetime('now'), datetime('now'));

-- 62 - Ingresos No Operacionales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '62', 'INGRESOS NO OPERACIONALES', 'income', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6205', 'Intereses Ganados', 'income', (SELECT id FROM account_categories WHERE code = '62'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6210', 'Dividendos', 'income', (SELECT id FROM account_categories WHERE code = '62'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6215', 'Arrendamientos', 'income', (SELECT id FROM account_categories WHERE code = '62'), 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '6220', 'Otros Ingresos', 'income', (SELECT id FROM account_categories WHERE code = '62'), 0, 1, datetime('now'), datetime('now'));

-- 63 - Devoluciones, Rebajas y Descuentos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES (gen_random_uuid(), '63', 'DEVOLUCIONES, REBAJAS Y DESCuentos', 'income', NULL, 0, 1, datetime('now'), datetime('now'));
