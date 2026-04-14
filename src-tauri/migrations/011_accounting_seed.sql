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
-- Note: Using code as id for simplicity (PUC codes are unique)

-- ============================================
-- 1xxx - ACTIVOS
-- ============================================
-- 11 - Caja y Bancos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1105', '1105', 'CAJA', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('110501', '110501', 'Caja General', 'asset', '1105', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('110502', '110502', 'Caja Menor', 'asset', '1105', 0, 1, datetime('now'), datetime('now'));

INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1110', '1110', 'BANCOS', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('111001', '111001', 'Banco de Colombia', 'asset', '1110', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('111002', '111002', 'Banco Davivienda', 'asset', '1110', 0, 1, datetime('now'), datetime('now'));

-- 12 - Inversiones Temporales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('12', '12', 'INVERSIONES TEMPORALES', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));

-- 13 - Cuentas por Cobrar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1305', '1305', 'CLIENTES', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('130501', '130501', 'Cuentas por Cobrar - Matrículas', 'asset', '1305', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('130502', '130502', 'Cuentas por Cobrar - Mensualidades', 'asset', '1305', 0, 1, datetime('now'), datetime('now'));

INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1380', '1380', 'OTRAS CUENTAS POR COBRAR', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));

-- 14 - Otras Cuentas por Cobrar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('14', '14', 'OTRAS CUENTAS POR COBRAR', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));

-- 15 - Inventarios
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('15', '15', 'INVENTARIOS', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1516', '1516', 'Materiales y Suministros', 'asset', '15', 0, 1, datetime('now'), datetime('now'));

-- 16 - Propiedad, Planta y Equipo
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('16', '16', 'PROPIEDAD,PLANTA Y EQUIPO', 'asset', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1615', '1615', 'Muebles y Enseres', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1616', '1616', 'Equipos de Oficina', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1617', '1617', 'Equipos de Computación', 'asset', '16', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('1620', '1620', 'Depreciación Acumulada', 'asset', '16', 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 2xxx - PASIVOS
-- ============================================
-- 21 - Proveedores
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('21', '21', 'PROVEEDORES', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2105', '2105', 'Proveedores Nacionales', 'liability', '21', 0, 1, datetime('now'), datetime('now'));

-- 22 - Cuentas por Pagar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('22', '22', 'CUENTAS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));

-- 23 - Obligaciones Laborales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('23', '23', 'OBLIGACIONES LABORALES', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2310', '2310', 'Salarios por Pagar', 'liability', '23', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2315', '2315', 'Cesantías Consignadas', 'liability', '23', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2320', '2320', 'Intereses sobre Cesantías', 'liability', '23', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2330', '2330', 'Prima de Servicios', 'liability', '23', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2340', '2340', 'Vacaciones', 'liability', '23', 0, 1, datetime('now'), datetime('now'));

-- 24 - Impuestos por Pagar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('24', '24', 'IMPUESTOS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2408', '2408', 'IVA por Pagar', 'liability', '24', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2412', '2412', 'Retención en la Fuente', 'liability', '24', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('2436', '2436', 'ReteIVA', 'liability', '24', 0, 1, datetime('now'), datetime('now'));

-- 25 - Obligaciones Financieras
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('25', '25', 'OBLIGACIONES FINANCIERAS', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));

-- 26 - Otras Cuentas por Pagar
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('26', '26', 'OTRAS CUENTAS POR PAGAR', 'liability', NULL, 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 3xxx - PATRIMONIO
-- ============================================
-- 31 - Capital
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('31', '31', 'CAPITAL', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('3115', '3115', 'Capital Suscrito y Pagado', 'equity', '31', 0, 1, datetime('now'), datetime('now'));

-- 32 - Reservas
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('32', '32', 'RESERVAS', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('3210', '3210', 'Reserva Legal', 'equity', '32', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('3220', '3220', 'Reservas Ocasionales', 'equity', '32', 0, 1, datetime('now'), datetime('now'));

-- 33 - Resultados del Ejercicio
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('33', '33', 'RESULTADOS DEL EJERCICIO', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('3310', '3310', 'Utilidades del Ejercicio', 'equity', '33', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('3320', '3320', 'Pérdidas del Ejercicio', 'equity', '33', 0, 1, datetime('now'), datetime('now'));

-- 34 - Resultados Acumulados
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('34', '34', 'RESULTADOS ACUMULADOS', 'equity', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('3410', '3410', 'Utilidades Acumuladas', 'equity', '34', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('3420', '3420', 'Pérdidas Acumuladas', 'equity', '34', 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 4xxx - GASTOS
-- ============================================
-- 41 - Gastos de Personal
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('41', '41', 'GASTOS DE PERSONAL', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4105', '4105', 'Salarios', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4110', '4110', 'Auxilio de Transporte', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4115', '4115', 'Horas Extras', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4120', '4120', 'Comisiones', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4130', '4130', 'Cesantías', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4135', '4135', 'Intereses sobre Cesantías', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4140', '4140', 'Prima de Servicios', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4145', '4145', 'Vacaciones', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4150', '4150', 'Aporte a Salud (EPS)', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4155', '4155', 'Aporte a Pensión (AFP)', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4160', '4160', 'Aporte a Riesgos Laborales (ARL)', 'expense', '41', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4165', '4165', 'Aporte a ICBF y SENA', 'expense', '41', 0, 1, datetime('now'), datetime('now'));

-- 42 - Gastos de Operación
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('42', '42', 'GASTOS DE OPERACIÓN', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4210', '4210', 'Arrendamientos', 'expense', '42', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4215', '4215', 'Servicios', 'expense', '42', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4220', '4220', 'Gas', 'expense', '42', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4225', '4225', 'Energía', 'expense', '42', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4230', '4230', 'Agua y Alcantarillado', 'expense', '42', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4235', '4235', 'Teléfono e Internet', 'expense', '42', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4240', '4240', 'Mantenimiento y Reparaciones', 'expense', '42', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4245', '4245', 'Gastos de Representación', 'expense', '42', 0, 1, datetime('now'), datetime('now'));

-- 43 - Depreciaciones
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('43', '43', 'DEPRECIACIONES', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));

-- 44 - Gastos Financieros
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('44', '44', 'GASTOS FINANCIEROS', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4410', '4410', 'Intereses', 'expense', '44', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4420', '4420', 'Comisiones Bancarias', 'expense', '44', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4430', '4430', 'GMF (4x1000)', 'expense', '44', 0, 1, datetime('now'), datetime('now'));

-- 45 - Otros Gastos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('45', '45', 'OTROS GASTOS', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4510', '4510', 'Gastos Extraordinarios', 'expense', '45', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4520', '4520', 'Gastos Varios', 'expense', '45', 0, 1, datetime('now'), datetime('now'));

-- 48 - Gastos por Impuestos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('48', '48', 'GASTOS POR IMPUESTOS', 'expense', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4810', '4810', 'Impuesto de Industria y Comercio (ICA)', 'expense', '48', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('4815', '4815', 'Gravamen a los Movimientos Financieros', 'expense', '48', 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 5xxx - COSTOS DE PRODUCCIÓN
-- ============================================
-- 51 - Costos de Operación
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('51', '51', 'COSTOS DE OPERACIÓN', 'cost', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('5105', '5105', 'Costos de Educación', 'cost', '51', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('5110', '5110', 'Materiales Didácticos', 'cost', '51', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('5115', '5115', 'Gastos de Personal Docente', 'cost', '51', 0, 1, datetime('now'), datetime('now'));

-- ============================================
-- 6xxx - INGRESOS
-- ============================================
-- 61 - Ingresos Operacionales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('61', '61', 'INGRESOS OPERACIONALES', 'income', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6105', '6105', 'SERVICIOS EDUCATIVOS', 'income', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6110', '6110', 'Matrículas', 'income', '6105', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6115', '6115', 'Mensualidades', 'income', '6105', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6120', '6120', 'Cursos Especiales', 'income', '6105', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6125', '6125', 'Certificaciones', 'income', '6105', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6130', '6130', 'Uniformes y Materiales', 'income', '6105', 0, 1, datetime('now'), datetime('now'));

-- 62 - Ingresos No Operacionales
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('62', '62', 'INGRESOS NO OPERACIONALES', 'income', NULL, 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6205', '6205', 'Intereses Ganados', 'income', '62', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6210', '6210', 'Dividendos', 'income', '62', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6215', '6215', 'Arrendamientos', 'income', '62', 0, 1, datetime('now'), datetime('now'));
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('6220', '6220', 'Otros Ingresos', 'income', '62', 0, 1, datetime('now'), datetime('now'));

-- 63 - Devoluciones, Rebajas y Descuentos
INSERT INTO account_categories (id, code, name, category_type, parent_id, balance, active, created_at, updated_at) 
VALUES ('63', '63', 'DEVOLUCIONES,REBAJAS Y DESCUNTOS', 'income', NULL, 0, 1, datetime('now'), datetime('now'));