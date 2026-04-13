# Tasks: Módulo de Contabilidad

## Resumen Ejecutivo

Breakdown de tareas profundas para implementar el módulo de contabilidad de Academix, incluyendo migraciones SQL, entidades y repositorios backend, comandos Tauri, componentes React, gráficos chart.js, PDFs jspdf, animaciones anime.js, skeleton loaders y rutas de navegación (solo admin).

---

## Fase 1: Infraestructura de Base de Datos

### 1.1 Migraciones SQL

- [ ] 1.1.1 Crear `src-tauri/migrations/010_accounting_schema.sql` con schema completo:
  - Tabla `employees` (id, user_id, document_type, document_number, first_name, last_name, email, phone, address, position, department, contract_type, base_salary, bank_name, bank_account, account_type, cci, afp, hire_date, termination_date, status, created_at, updated_at)
  - Tabla `payroll_runs` (id, period_start, period_end, status, total_gross, total_deductions, total_net, created_at, created_by)
  - Tabla `payroll_entries` (id, payroll_run_id, employee_id, base_salary, hours_worked, overtime_hours, overtime_amount, bonuses, commissions, mobility, food, other_income, afp_deduction, onp_deduction, essalud, itf, other_deductions, gross_income, net_income, status, created_at)
  - Tabla `accounting_entries` (id, date, reference, description, debit_account, credit_account, amount, entry_type, related_id, related_type, created_at, created_by)
  - Tabla `account_categories` (id, code, name, type, parent_id, balance, active, created_at, updated_at)
  - Tabla `invoices` (id, series, number, client_name, client_ruc, client_address, emission_date, due_date, subtotal, igv, total, status, payment_method, paid_date, created_at, created_by)
  - Tabla `invoice_lines` (id, invoice_id, description, quantity, unit_price, total, created_at)
- [ ] 1.1.2 Crear `src-tauri/migrations/011_accounting_seed.sql` con plan de cuentas estándar peruano:
  - 1xxx Activos (10101 Efectivo, 10401 Banco, 1201 Cuentas por cobrar)
  - 2xxx Pasivos (20101 Proveedores, 4011 Remuneraciones por pagar)
  - 3xxx Patrimonio (301 Capital, 302 Reservas)
  - 4xxx Gastos (401 Sueldos, 403 Servicios, 621 Sueldos)
  - 5xxx Costos (501 Costos de servicios)
  - 6xxx Ingresos (701 Servicios educativos)

---

## Fase 2: Backend - Dominio y Entidades

### 2.1 Entidades del Dominio

- [ ] 2.1.1 Crear `src-tauri/src/domain/entities/employee.rs`:
  - Struct `Employee` con todos los campos definidos en design
  - Impl `Entity` para timestamp management
  - Método `new()` y `update()`
- [ ] 2.1.2 Crear `src-tauri/src/domain/entities/payroll.rs`:
  - Struct `PayrollRun` con período, estado, totales
  - Struct `PayrollEntry` con todos los componentes salariales
  - Métodos para cálculo de neto
- [ ] 2.1.3 Crear `src-tauri/src/domain/entities/accounting.rs`:
  - Struct `AccountingEntry` (asiento contable)
  - Struct `AccountCategory` (cuenta del plan)
  - Enum `EntryType` (manual, automatic, adjustment)
- [ ] 2.1.4 Crear `src-tauri/src/domain/entities/invoice.rs`:
  - Struct `Invoice` con campos de facturación
  - Struct `InvoiceLine` para detalle
  - Enum `InvoiceStatus` y `PaymentMethod`
- [ ] 2.1.5 Modificar `src-tauri/src/domain/entities/mod.rs` para exportar nuevas entidades

### 2.2 Puertos (Interfaces)

- [ ] 2.2.1 Crear `src-tauri/src/application/ports/employee.rs`:
  - Trait `EmployeeRepository` con métodos: create, get_by_id, list, update, delete
- [ ] 2.2.2 Crear `src-tauri/src/application/ports/payroll.rs`:
  - Trait `PayrollRepository` para PayrollRun
  - Trait `PayrollEntryRepository` para PayrollEntry
- [ ] 2.2.3 Crear `src-tauri/src/application/ports/accounting.rs`:
  - Trait `AccountingEntryRepository`
  - Trait `AccountCategoryRepository`
- [ ] 2.2.4 Crear `src-tauri/src/application/ports/invoice.rs`:
  - Trait `InvoiceRepository` con métodos CRUD
- [ ] 2.2.5 Modificar `src-tauri/src/application/ports/mod.rs` para exportar puertos

### 2.3 DTOs

- [x] 2.3.1 Crear `src-tauri/src/application/dto/employee.rs`:
  - Struct `EmployeeDto` para responses
  - Struct `CreateEmployeeRequest` para create
  - Struct `UpdateEmployeeRequest` para update
- [x] 2.3.2 Crear `src-tauri/src/application/dto/payroll.rs`:
  - Struct `PayrollRunDto`
  - Struct `PayrollEntryDto`
  - Struct `RunPayrollRequest` (periodo, employee_ids)
- [x] 2.3.3 Crear `src-tauri/src/application/dto/accounting.rs`:
  - Struct `AccountingEntryDto`
  - Struct `AccountCategoryDto`
  - Struct `CreateEntryRequest`
- [x] 2.3.4 Crear `src-tauri/src/application/dto/invoice.rs`:
  - Struct `InvoiceDto`
  - Struct `CreateInvoiceRequest`
  - Struct `InvoiceLineDto`
- [x] 2.3.5 Modificar `src-tauri/src/application/dto/mod.rs` para exportar DTOs

---

## Fase 3: Backend - Repositorios SQLite

### 3.1 Implementación de Repositorios

- [x] 3.1.1 Crear `src-tauri/src/infrastructure/repositories/employee.rs`:
  - Trait `SqliteEmployeeRepository` que implementa `EmployeeRepository`
- [x] 3.1.2 Crear `src-tauri/src/infrastructure/repositories/payroll.rs`:
  - Trait `SqlitePayrollRepository` para PayrollRun
  - Trait `SqlitePayrollEntryRepository` para PayrollEntry
- [x] 3.1.3 Crear `src-tauri/src/infrastructure/repositories/accounting.rs`:
  - Trait `SqliteAccountingEntryRepository`
  - Trait `SqliteAccountCategoryRepository`
- [x] 3.1.4 Crear `src-tauri/src/infrastructure/repositories/invoice.rs`:
  - Trait `SqliteInvoiceRepository`
- [x] 3.1.5 Modificar `src-tauri/src/infrastructure/repositories/mod.rs` para exportar traits

### 3.2 Implementación SQLite

- [ ] 3.2.1 Crear `src-tauri/src/infrastructure/repositories/sqlite/employee.rs`:
  - Struct `SqliteEmployeeRepositoryImpl`
  - Implementar todos los métodos del trait
- [ ] 3.2.2 Crear `src-tauri/src/infrastructure/repositories/sqlite/payroll.rs`:
  - Struct `SqlitePayrollRepositoryImpl`
  - Struct `SqlitePayrollEntryRepositoryImpl`
- [ ] 3.2.3 Crear `src-tauri/src/infrastructure/repositories/sqlite/accounting.rs`:
  - Struct `SqliteAccountingRepositoryImpl`
  - Struct `SqliteCategoryRepositoryImpl`
- [ ] 3.2.4 Crear `src-tauri/src/infrastructure/repositories/sqlite/invoice.rs`:
  - Struct `SqliteInvoiceRepositoryImpl`
- [ ] 3.2.5 Modificar `src-tauri/src/infrastructure/repositories/sqlite/mod.rs` para exports

---

## Fase 4: Backend - Use Cases (Lógica de Negocio)

### 4.1 Servicios / Use Cases

- [x] 4.1.1 Crear `src-tauri/src/application/use_cases/employee.rs`:
  - Struct `EmployeeService<R: EmployeeRepository>`
  - Métodos: create_employee, get_employee, list_employees, update_employee, delete_employee
  - Validación de campos obligatorios
- [x] 4.1.2 Crear `src-tauri/src/application/use_cases/payroll.rs`:
  - Struct `PayrollService<R: PayrollRepository, E: PayrollEntryRepository, Emp: EmployeeRepository>`
  - Método `run_payroll()` con cálculo de AFP (11.25-11.60%), Essalud (9%), ITF (0.5% sobre depósitos > S/3500)
  - Métodos: generate_receipt_pdf, generate_all_receipts
  - Constantes: AFP_RATES, ESSALUD_RATE, ITF_RATE, ITF_THRESHOLD
- [x] 4.1.3 Crear `src-tauri/src/application/use_cases/accounting.rs`:
  - Struct `AccountingService<R: AccountingEntryRepository, C: AccountCategoryRepository>`
  - Métodos: create_entry, list_entries, get_trial_balance, get_income_statement
  - Generación automática de asientos desde nómina
- [x] 4.1.4 Crear `src-tauri/src/application/use_cases/invoice.rs`:
  - Struct `InvoiceService<R: InvoiceRepository>`
  - Métodos: create_invoice, get_invoice, list_invoices, export_pdf
  - Cálculo automático de IGV (18%)
- [x] 4.1.5 Modificar `src-tauri/src/application/use_cases/mod.rs` para exportar use cases

---

## Fase 5: Backend - Comandos Tauri

### 5.1 Comandos de Empleados

- [x] 5.1.1 Crear `src-tauri/src/commands/employees.rs`:
  - Comando `create_employee`: recibe CreateEmployeeRequest, retorna EmployeeDto
  - Comando `get_employee`: recibe id, retorna EmployeeDto
  - Comando `list_employees`: recibe filtros opcionales (status, department), retorna Vec<EmployeeDto>
  - Comando `update_employee`: recibe id + UpdateEmployeeRequest, retorna EmployeeDto
  - Comando `delete_employee`: recibe id, retorna bool

### 5.2 Comandos de Nómina

- [x] 5.2.1 Crear `src-tauri/src/commands/payroll.rs`:
  - Comando `run_payroll`: recibe RunPayrollRequest, retorna PayrollRunDto con Entries
  - Comando `get_payroll_run`: recibe id, retorna PayrollRunDto con entries
  - Comando `list_payroll_runs`: retorna Vec<PayrollRunDto>
  - Comando `generate_payroll_receipt`: recibe payroll_entry_id, retorna PDF bytes (base64)
  - Comando `generate_all_receipts`: recibe payroll_run_id, retorna ZIP con PDFs

### 5.3 Comandos de Contabilidad

- [x] 5.3.1 Crear `src-tauri/src/commands/accounting.rs`:
  - Comando `create_entry`: recibe CreateEntryRequest, retorna AccountingEntryDto
  - Comando `get_entry`: recibe id, retorna AccountingEntryDto
  - Comando `list_entries`: recibe filtros (date_from, date_to, account), retorna Vec<AccountingEntryDto>
  - Comando `get_trial_balance`: recibe fecha_corte, retorna balance de comprobación
  - Comando `get_income_statement`: recibe periodo (start, end), retorna estado de resultados

### 5.4 Comandos de Facturación

- [x] 5.4.1 Crear `src-tauri/src/commands/invoices.rs`:
  - Comando `create_invoice`: recibe CreateInvoiceRequest, retorna InvoiceDto
  - Comando `get_invoice`: recibe id, retorna InvoiceDto
  - Comando `list_invoices`: recibe filtros (status, client), retorna Vec<InvoiceDto>
  - Comando `export_invoice_pdf`: recibe invoice_id, retorna PDF bytes
  - Comando `register_payment`: recibe invoice_id + payment info, actualiza estado

### 5.5 Registro de Comandos

- [x] 5.5.1 Modificar `src-tauri/src/commands/mod.rs` para exportar todos los módulos
- [x] 5.5.2 Modificar `src-tauri/src/lib.rs` para registrar comandos con `#[tauri::command]`

---

## Fase 6: Frontend - Configuración y Tipos

### 6.1 Dependencias

- [ ] 6.1.1 Instalar `chart.js` y `react-chartjs-2` para gráficos
- [ ] 6.1.2 Instalar `jspdf` y `jspdf-autotable` para PDFs
- [ ] 6.1.3 Instalar `animejs` para animaciones

### 6.2 Tipos TypeScript

- [ ] 6.2.1 Crear `src/features/accounting/types/accounting.ts`:
  - Interfaces: Employee, PayrollRun, PayrollEntry, AccountingEntry, AccountCategory, Invoice, InvoiceLine
  - Enums: DocumentType, ContractType, AFP, AccountType, EntryType, InvoiceStatus, PaymentMethod, CategoryType
  - Types para requests: CreateEmployeeRequest, RunPayrollRequest, CreateEntryRequest, CreateInvoiceRequest

### 6.3 Hooks Personalizados

- [ ] 6.3.1 Crear `src/features/accounting/hooks/useEmployees.ts`:
  - Funciones: createEmployee, getEmployee, listEmployees, updateEmployee, deleteEmployee
  - Estados: loading, error, data
  - Uso de `invoke()` de Tauri
- [ ] 6.3.2 Crear `src/features/accounting/hooks/usePayroll.ts`:
  - Funciones: runPayroll, getPayrollRun, listPayrollRuns, generateReceipt, generateAllReceipts
- [ ] 6.3.3 Crear `src/features/accounting/hooks/useAccounting.ts`:
  - Funciones: createEntry, getEntry, listEntries, getTrialBalance, getIncomeStatement
- [ ] 6.3.4 Crear `src/features/accounting/hooks/useReports.ts`:
  - Funciones: getMonthlyReport, getAnnualReport, exportToPDF, exportToExcel
- [ ] 6.3.5 Crear `src/features/accounting/hooks/useInvoices.ts`:
  - Funciones: createInvoice, getInvoice, listInvoices, exportInvoicePDF, registerPayment

### 6.4 Skeleton Loader Base

- [ ] 6.4.1 Crear `src/shared/ui/components/Skeleton.tsx`:
  - Componente Skeleton reutilizable
  - Props: width, height, borderRadius, className
  - Animación de pulse con CSS keyframes

---

## Fase 7: Frontend - Skeleton Loaders del Módulo

### 7.1 Componentes Skeleton

- [ ] 7.1.1 Crear `src/features/accounting/components/Skeletons.tsx`:
  - `TableSkeleton`: skeleton para tablas (filas x columnas)
  - `CardSkeleton`: skeleton para cards de resumen
  - `FormSkeleton`: skeleton para formularios
  - `ChartSkeleton`: skeleton para gráficos (rectángulo con animación)
  - `InvoiceSkeleton`: skeleton específico para facturas
- [ ] 7.1.2 Implementar animaciones de skeleton con CSS:
  - Efecto shimmer/gradient animado
  - Timing configurables (300ms, 500ms, 800ms)

---

## Fase 8: Frontend - Tablas

### 8.1 Tablas de Datos

- [ ] 8.1.1 Crear `src/features/accounting/components/EmployeesTable.tsx`:
  - Columnas: Documento, Nombre, Cargo, Departamento, Contracto, Salario, Estado, Acciones
  - Features: sortable, paginated, search/filter
  - Integrar con useEmployees hook
  - Skeleton loading states
- [ ] 8.1.2 Crear `src/features/accounting/components/PayrollTable.tsx`:
  - Columnas: Empleado, Salario Base, Horas Extra, Bonificaciones, AFP, Essalud, ITF, Neto
  - Feature: expandable rows para ver detalle
  - Integrar con usePayroll hook
- [ ] 8.1.3 Crear `src/features/accounting/components/EntriesTable.tsx`:
  - Columnas: Fecha, Referencia, Descripción, Cuenta Debe, Cuenta Haber, Monto, Tipo
  - Feature: filtrado por fecha, account
  - Integrar con useAccounting hook

---

## Fase 9: Frontend - Formularios

### 9.1 Formularios del Módulo

- [ ] 9.1.1 Crear `src/features/accounting/components/EmployeeForm.tsx`:
  - Campos: datos personales (nombre, documento, email), laborales (cargo, departamento, contrato, salary), bancarios (banco, cuenta, CCI, AFP), fechas (ingreso, termination)
  - Validación con Zod
  - Modo create/update
  - Animación con anime.js en submit success
- [ ] 9.1.2 Crear `src/features/accounting/components/PayrollRunForm.tsx`:
  - Selector de período (mes/año)
  - Lista de empleados a incluir (checkboxes)
  - Botón "Ejecutar Nómina"
  - Loading state con skeleton
- [ ] 9.1.3 Crear `src/features/accounting/components/EntryForm.tsx`:
  - Campos: fecha, cuenta debe (dropdown), cuenta haber (dropdown), monto, descripción, referencia
  - Validación: debe != haber, monto > 0
- [ ] 9.1.4 Crear `src/features/accounting/components/InvoiceForm.tsx`:
  - Datos cliente: nombre, RUC, dirección
  - Líneas: descripción, cantidad, precio unitario (dynamic add/remove)
  - Cálculo automático: subtotal, IGV (18%), total

---

## Fase 10: Frontend - Gráficos con Chart.js

### 10.1 Componentes de Gráficos

- [ ] 10.1.1 Crear `src/features/accounting/components/PayrollChart.tsx`:
  - Gráfico de línea: tendencia salarial últimos 12 meses
  - Gráfico de barras: distribución por departamento
  - Configurar con react-chartjs-2
  - Animación de entrada con anime.js (fade in + scale)
- [ ] 10.1.2 Crear `src/features/accounting/components/ExpenseChart.tsx`:
  - Gráfico doughnut: composición de gastos (sueldos, servicios, impuestos)
  - Gráfico de barras apiladas: gastos mensuales
  - Colores profesionales (azules, verdes, grises)
- [ ] 10.1.3 Crear `src/features/accounting/components/RevenueChart.tsx`:
  - Gráfico de línea: ingresos por servicios educativos
  - Gráfico de barras: comparación mensual (presupuesto vs real)
- [ ] 10.1.4 Configurar Chart.js global:
  - Default colors, fonts, responsive settings
  - Tooltips customizados

---

## Fase 11: Frontend - PDFs con jsPDF

### 11.1 Generación de PDFs

- [ ] 11.1.1 Crear `src/features/accounting/utils/pdfGenerator.ts`:
  - Función `generatePayrollReceiptPDF(entry: PayrollEntry): jsPDF`
  - Función `generateAllReceiptsPDF(run: PayrollRun): jsPDF`
  - Función `generateInvoicePDF(invoice: Invoice): jsPDF`
  - Función `generateReportPDF(report: MonthlyReport): jsPDF`
- [ ] 11.1.2 Implementar template de recibo de nómina:
  - Header: empresa, período
  - Sección ingresos: desglose por tipo
  - Sección descuentos: AFP, Essalud, ITF, otros
  - Pie: total neto, firma
- [ ] 11.1.3 Implementar template de factura:
  - Encabezado: serie, número, fecha
  - Cliente: nombre, RUC, dirección
  - Tabla de líneas: descripción, cantidad, precio, total
  - Totales: subtotal, IGV, total
  - Pie: condiciones de pago

---

## Fase 12: Frontend - Animaciones con Anime.js

### 12.1 Animaciones del Módulo

- [ ] 12.1.1 Crear `src/features/accounting/utils/animations.ts`:
  - Función `animateTableRowIn(index: number)`: stagger animation para filas de tabla
  - Función `animateCardIn(card: HTMLElement)`: scale + fade para cards de dashboard
  - Función `animateChartIn(chart: HTMLElement)`: fade + slide para gráficos
  - Función `animateFormSubmit(form: HTMLElement)`: success animation
- [ ] 12.1.2 Implementar animaciones de transición:
  - Page transitions: fade + slide entre páginas
  - Modal animations: scale + fade in/out
  - Toast notifications: slide from right
- [ ] 12.1.3 Integrar con skeleton loaders:
  - Animación de salida del skeleton (fade out)
  - Animación de entrada del contenido (fade in + slide)

---

## Fase 13: Frontend - Páginas/Rutas

### 13.1 Páginas del Módulo

- [ ] 13.1.1 Crear `src/features/accounting/routes/EmployeesPage.tsx`:
  - Layout: header con título + botón "Nuevo Empleado"
  - Sección: tabla de empleados con search/filter
  - Modal: EmployeeForm para create/update
  - Animación de entrada con anime.js
- [ ] 13.1.2 Crear `src/features/accounting/routes/PayrollPage.tsx`:
  - Layout: header + PayrollRunForm
  - Sección: PayrollTable con payroll runs
  - Acciones: ejecutar nómina, ver detalles, generar recibos
- [ ] 13.1.3 Crear `src/features/accounting/routes/AccountingPage.tsx`:
  - Layout: tabs para libro diario, balance, estado resultados
  - Tab 1: EntriesTable con filtros de fecha
  - Tab 2: Balance de comprobación
  - Tab 3: Estado de resultados
- [ ] 13.1.4 Crear `src/features/accounting/routes/ReportsPage.tsx`:
  - Layout: selector de tipo (mensual/anual) + período
  - Dashboard: charts (PayrollChart, ExpenseChart, RevenueChart)
  - Acciones: exportar PDF, exportar Excel
- [ ] 13.1.5 Crear `src/features/accounting/routes/InvoicesPage.tsx`:
  - Layout: tabla de facturas con filtros (estado, cliente)
  - Acciones: crear factura, ver PDF, registrar pago

### 13.2 Configuración de Rutas

- [ ] 13.2.1 Modificar `src/app/router.tsx`:
  - Agregar ruta `/accounting/employees` (protegida: admin only)
  - Agregar ruta `/accounting/payroll` (protegida: admin only)
  - Agregar ruta `/accounting/accounting` (protegida: admin only)
  - Agregar ruta `/accounting/reports` (protegida: admin only)
  - Agregar ruta `/accounting/invoices` (protegida: admin only)
- [ ] 13.2.2 Crear `src/features/accounting/index.ts` como barrel export

---

## Fase 14: Testing y Verificación

### 14.1 Tests Unitarios (Rust)

- [ ] 14.1.1 Tests de cálculo de nómina en `src-tauri/src/application/use_cases/payroll.rs`:
  - `test_afp_deduction_prima()`: verificar 11.25%
  - `test_afp_deduction_habitat()`: verificar 11.35%
  - `test_afp_deduction_profuturo()`: verificar 11.60%
  - `test_essalud_calculation()`: verificar 9%
  - `test_itf_applies_above_threshold()`: verificar 0.5% sobre >3500
  - `test_itf_not_applies_below_threshold()`: verificar 0% si <=3500
  - `test_net_salary_calculation()`: verificar ingreso neto

### 14.2 Tests de Integración

- [ ] 14.2.1 Tests de integración de empleados:
  - Crear empleado, listar, actualizar, eliminar
- [ ] 14.2.2 Tests de integración de nómina:
  - Ejecutar nómina completa, verificar totales
  - Generar recibo PDF, verificar contenido

### 14.3 Tests E2E (Playwright)

- [ ] 14.3.1 Test: crear empleado desde UI
- [ ] 14.3.2 Test: ejecutar nómina y verificar resultados
- [ ] 14.3.3 Test: generar reporte y verificar PDF descargado
- [ ] 14.3.4 Test: crear factura y registrar pago

---

## Fase 15: Documentación y Limpieza

### 15.1 Documentación

- [ ] 15.1.1 Agregar comentarios a comandos Tauri
- [ ] 15.1.2 Documentar API de hooks en JSDoc
- [ ] 15.1.3 Actualizar README del módulo con ejemplos de uso

### 15.2 Limpieza

- [ ] 15.2.1 Remover código de debug/temporary
- [ ] 15.2.2 Verificar que no haya console.log en producción
- [ ] 15.2.3 Ejecutar type check completo

---

## Resumen de Tareas

| Fase | Descripción | Tareas |
|------|-------------|--------|
| 1 | Infraestructura DB | 2 |
| 2 | Backend - Dominio | 10 |
| 3 | Backend - Repositorios | 9 |
| 4 | Backend - Use Cases | 5 |
| 5 | Backend - Comandos Tauri | 13 |
| 6 | Frontend - Config | 10 |
| 7 | Frontend - Skeletons | 2 |
| 8 | Frontend - Tablas | 3 |
| 9 | Frontend - Formularios | 4 |
| 10 | Frontend - Gráficos | 4 |
| 11 | Frontend - PDFs | 3 |
| 12 | Frontend - Animaciones | 3 |
| 13 | Frontend - Rutas | 15 |
| 14 | Testing | 7 |
| 15 | Documentación | 3 |
| **Total** | | **96** |

---

## Orden de Implementación Recomendado

1. **Semana 1**: Fases 1-5 (Backend completo)
2. **Semana 2**: Fases 6-9 (Frontend core)
3. **Semana 3**: Fases 10-12 (Visualización)
4. **Semana 4**: Fases 13-15 (Rutas, testing, polish)

**Dependencias críticas**:
- Migraciones deben existir antes de repositorios
- Entidades/DTOs deben existir antes de use cases
- Use cases deben existir antes de comandos
- Hooks deben existir antes de componentes
- Componentes base (skeletons, tablas) antes de páginas

---

## Riesgos Identificados

- **R1**: Cálculos de nómina con reglas complejas podrían requerir ajustes
- **R2**: PDF generation podría variar según requerimientos de layout
- **R3**: Animaciones con anime.js podrían necesitar fine-tuning
- **R4**: Testing de PDFs requiere validación de contenido

**Mitigación**: Priorizar testing de cálculo de nómina primero; prototipar PDFs y animaciones temprano.