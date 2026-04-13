# Design: Módulo de Contabilidad

## Enfoque Técnico

Se implementará el módulo de contabilidad con arquitectura hexagonal en Rust/Tauri para el backend, y React 19 para el frontend. El módulo soporta gestión de empleados, cálculo automático de nómina con descuentos legales peruanos (AFP, Essalud, ONP, ITF), generación de asientos contables, facturación de servicios educativos, y reportes financieros con gráficos y PDFs.

La persistencia será en SQLite, manteniendo la estructura hexagonal del proyecto existente (domain/entities, application/ports, application/use_cases, infrastructure/repositories, commands).

---

## Decisiones de Arquitectura

### Decisión: Estructura de Entidades de Contabilidad

**Elección**: Crear entidades separadas para employees, payroll, accounting_entries y categories en el dominio.

**Alternativas consideradas**: Reutilizar users existente, agregar campos de nómina a users.

**Rationale**: El alcance de contabilidad (payroll, asientos, categorías) requiere datos específicos que no tienen sentido en users. Mantener separación facilita mantenimiento y futuras migraciones. La entidad Employee es distinta conceptualmente de User (roles del sistema vs. empleados para nómina).

### Decisión: Patrón de Servicios con Genéricos

**Elección**: Seguir el patrón de StudentService<R: StudentRepository, G: GroupRepository> con genéricos para inyección de repositorios.

**Alternativas consideradas**: Servicios concretos sin genéricos, repositorios singletons.

**Rationale**: Permite testing con mocks, sigue el patrón del proyecto existente (student, group, course services), facilita cambios de implementación.

### Decisión: Bibliotecas de Visualización

**Elección**: 
- **Gráficos**: chart.js con react-chartjs-2 (no recharts)
- **PDFs**: jspdf con jspdf-autotable
- **Animaciones**: anime.js

**Alternativas consideradas**: 
- recharts: Más popular en React pero chart.js tiene mejor soporte para gráficos financieros complejos (balance, comparación)
- pdfmake: Menos flexible para personalización de recibos
- framer-motion: Ya usado en proyecto pero anime.js ofrece más control para animaciones complejas en dashboards

**Rationale**: 
- chart.js tiene mejor documentación para gráficos de Payroll (líneas de tendencia salarial, barras de gastos por departamento)
- jspdf es más ligero y permite mayor control sobre el layout de recibos de pago
- anime.js proporciona animaciones declarativas que se integran bien con los skeletons loaders

### Decisión: Estados de Carga con Skeleton Loaders

**Elección**: Crear componentes skeleton para cada tipo de dato (tabla, card, formulario, gráfico).

**Alternativas consideradas**: Spinners simples, skeletons genéricos.

**Rationale**: Mejora percibida de performance, consistencia visual con el resto de la app, reduce percepciones de espera.

---

## Flujo de Datos

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (React 19)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  Routes: /accounting/employees, /accounting/payroll, /accounting/entries   │
│           /accounting/reports, /accounting/invoices                        │
│                                                                             │
│  Hooks: useEmployees(), usePayroll(), useAccountingEntries(),              │
│         useInvoices(), useReports()                                         │
│                                                                             │
│  Components:                                                               │
│    - Pages: EmployeesPage, PayrollPage, EntriesPage, ReportsPage          │
│    - Forms: EmployeeForm, PayrollRunForm, EntryForm, InvoiceForm          │
│    - Tables: EmployeesTable, PayrollTable, EntriesTable                  │
│    - Charts: PayrollChart, ExpenseChart, RevenueChart                     │
│    - Skeletons: TableSkeleton, CardSkeleton, ChartSkeleton               │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    invoke() / @tauri-apps/api/core
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           BACKEND (Tauri/Rust)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  Commands (src/commands/):                                                 │
│    - employees.rs: create_employee, get_employee, list_employees,        │
│                    update_employee, delete_employee                        │
│    - payroll.rs: run_payroll, get_payroll_run, list_payroll_runs,         │
│                   generate_payroll_receipt, generate_all_receipts          │
│    - accounting.rs: create_entry, get_entry, list_entries,                │
│                      get_trial_balance, get_income_statement               │
│    - invoices.rs: create_invoice, get_invoice, list_invoices,             │
│                    export_invoice_pdf                                      │
│                                                                             │
│  Use Cases (src/application/use_cases/):                                   │
│    - employee.rs: EmployeeService                                         │
│    - payroll.rs: PayrollService (cálculo AFP, Essalud, ITF, neto)        │
│    - accounting.rs: AccountingService (asientos, libro diario)            │
│    - invoice.rs: InvoiceService                                           │
│                                                                             │
│  Ports (src/application/ports/):                                           │
│    - employee.rs: EmployeeRepository                                       │
│    - payroll.rs: PayrollRepository, PayrollEntryRepository                │
│    - accounting.rs: AccountingEntryRepository, AccountCategoryRepository  │
│    - invoice.rs: InvoiceRepository                                         │
│                                                                             │
│  Domain (src/domain/entities/):                                            │
│    - employee.rs: Employee                                                │
│    - payroll.rs: PayrollRun, PayrollEntry                                  │
│    - accounting.rs: AccountingEntry, AccountCategory                      │
│    - invoice.rs: Invoice, InvoiceLine                                     │
│                                                                             │
│  Infrastructure (src/infrastructure/repositories/sqlite/):                 │
│    - employee.rs: SqliteEmployeeRepository                                  │
│    - payroll.rs: SqlitePayrollRepository                                    │
│    - accounting.rs: SqliteAccountingRepository, SqliteCategoryRepository  │
│    - invoice.rs: SqliteInvoiceRepository                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          DATABASE (SQLite)                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│  Tablas:                                                                   │
│    - employees: id, user_id, document_type, document_number, name,         │
│                 email, phone, address, position, department,              │
│                 contract_type, base_salary, bank_name, bank_account,       │
│                 account_type, cci, afp, hire_date, termination_date,      │
│                 status, created_at, updated_at                             │
│    - payroll_runs: id, period_start, period_end, status, total_gross,     │
│                     total_deductions, total_net, created_at, created_by  │
│    - payroll_entries: id, payroll_run_id, employee_id, base_salary,      │
│                        hours_worked, overtime_hours, overtime_amount,      │
│                        bonuses, commissions, mobility, food, other_income, │
│                        afp_deduction, onp_deduction, essalud, itf,        │
                        other_deductions, gross_income, net_income,         │
│                        status, created_at                                  │
│    - accounting_entries: id, date, reference, description,               │
│                           debit_account, credit_account, amount,          │
│                           entry_type, related_id, related_type,           │
│                           created_at, created_by                          │
│    - account_categories: id, code, name, type, parent_id,                 │
│                           balance, active, created_at, updated_at        │
│    - invoices: id, series, number, client_name, client_ruc,               │
│                client_address, emission_date, due_date, subtotal,        │
│                igv, total, status, payment_method, paid_date,            │
│                created_at, created_by                                     │
│    - invoice_lines: id, invoice_id, description, quantity, unit_price,   │
│                     total, created_at                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Cambios de Archivos

### Backend - Rust (src-tauri/)

| Archivo | Acción | Descripción |
|---------|--------|-------------|
| `src/domain/entities/mod.rs` | Modificar | Agregar exports de nuevas entidades |
| `src/domain/entities/employee.rs` | Crear | Entidad Employee con campos de nómina |
| `src/domain/entities/payroll.rs` | Crear | Entidades PayrollRun, PayrollEntry |
| `src/domain/entities/accounting.rs` | Crear | Entidades AccountingEntry, AccountCategory |
| `src/domain/entities/invoice.rs` | Crear | Entidades Invoice, InvoiceLine |
| `src/application/ports/mod.rs` | Modificar | Agregar exports de nuevos puertos |
| `src/application/ports/employee.rs` | Crear | Puerto EmployeeRepository |
| `src/application/ports/payroll.rs` | Crear | Puertos PayrollRepository, PayrollEntryRepository |
| `src/application/ports/accounting.rs` | Crear | Puertos AccountingEntryRepository, AccountCategoryRepository |
| `src/application/ports/invoice.rs` | Crear | Puerto InvoiceRepository |
| `src/application/dto/mod.rs` | Modificar | Agregar exports de nuevos DTOs |
| `src/application/dto/employee.rs` | Crear | DTOs para Employee |
| `src/application/dto/payroll.rs` | Crear | DTOs para Payroll |
| `src/application/dto/accounting.rs` | Crear | DTOs para Accounting |
| `src/application/dto/invoice.rs` | Crear | DTOs para Invoice |
| `src/application/use_cases/mod.rs` | Modificar | Agregar exports de nuevos use cases |
| `src/application/use_cases/employee.rs` | Crear | EmployeeService |
| `src/application/use_cases/payroll.rs` | Crear | PayrollService con cálculo AFP/Essalud/ITF |
| `src/application/use_cases/accounting.rs` | Crear | AccountingService |
| `src/application/use_cases/invoice.rs` | Crear | InvoiceService |
| `src/infrastructure/repositories/mod.rs` | Modificar | Agregar exports |
| `src/infrastructure/repositories/employee.rs` | Crear | Puerto trait |
| `src/infrastructure/repositories/payroll.rs` | Crear | Puerto trait |
| `src/infrastructure/repositories/accounting.rs` | Crear | Puerto trait |
| `src/infrastructure/repositories/invoice.rs` | Crear | Puerto trait |
| `src/infrastructure/repositories/sqlite/mod.rs` | Modificar | Agregar exports |
| `src/infrastructure/repositories/sqlite/employee.rs` | Crear | SqliteEmployeeRepository |
| `src/infrastructure/repositories/sqlite/payroll.rs` | Crear | SqlitePayrollRepository |
| `src/infrastructure/repositories/sqlite/accounting.rs` | Crear | SqliteAccountingRepository, SqliteCategoryRepository |
| `src/infrastructure/repositories/sqlite/invoice.rs` | Crear | SqliteInvoiceRepository |
| `src/commands/mod.rs` | Modificar | Agregar exports de nuevos módulos |
| `src/commands/employees.rs` | Crear | Comandos CRUD para empleados |
| `src/commands/payroll.rs` | Crear | Comandos para nómina |
| `src/commands/accounting.rs` | Crear | Comandos para asientos contables |
| `src/commands/invoices.rs` | Crear | Comandos para facturas |
| `src/lib.rs` | Modificar | Registrar nuevos comandos |

### Base de Datos

| Archivo | Acción | Descripción |
|---------|--------|-------------|
| `migrations/010_accounting_schema.sql` | Crear | Migración con todas las tablas contables |
| `migrations/011_accounting_seed.sql` | Crear | Seed del plan de cuentas estándar |

### Frontend - React (src/)

| Archivo | Acción | Descripción |
|---------|--------|-------------|
| `src/features/accounting/index.ts` | Crear | Export barrel del módulo |
| `src/features/accounting/routes/EmployeesPage.tsx` | Crear | Página de gestión de empleados |
| `src/features/accounting/routes/PayrollPage.tsx` | Crear | Página de nómina |
| `src/features/accounting/routes/AccountingPage.tsx` | Crear | Página de asientos contables |
| `src/features/accounting/routes/ReportsPage.tsx` | Crear | Página de reportes |
| `src/features/accounting/routes/InvoicesPage.tsx` | Crear | Página de facturación |
| `src/features/accounting/components/EmployeeForm.tsx` | Crear | Formulario de empleado |
| `src/features/accounting/components/PayrollRunForm.tsx` | Crear | Formulario para ejecutar nómina |
| `src/features/accounting/components/EntryForm.tsx` | Crear | Formulario de asiento contable |
| `src/features/accounting/components/InvoiceForm.tsx` | Crear | Formulario de factura |
| `src/features/accounting/components/EmployeesTable.tsx` | Crear | Tabla de empleados |
| `src/features/accounting/components/PayrollTable.tsx` | Crear | Tabla de nómina |
| `src/features/accounting/components/EntriesTable.tsx` | Crear | Tabla de asientos |
| `src/features/accounting/components/PayrollChart.tsx` | Crear | Gráfico de nómina (chart.js) |
| `src/features/accounting/components/ExpenseChart.tsx` | Crear | Gráfico de gastos |
| `src/features/accounting/components/Skeletons.tsx` | Crear | Skeleton loaders para el módulo |
| `src/features/accounting/hooks/useEmployees.ts` | Crear | Hook para empleados |
| `src/features/accounting/hooks/usePayroll.ts` | Crear | Hook para nómina |
| `src/features/accounting/hooks/useAccounting.ts` | Crear | Hook para asientos contables |
| `src/features/accounting/hooks/useReports.ts` | Crear | Hook para reportes |
| `src/features/accounting/hooks/useInvoices.ts` | Crear | Hook para facturas |
| `src/features/accounting/types/accounting.ts` | Crear | Tipos TypeScript |
| `src/app/router.tsx` | Modificar | Agregar rutas del módulo |
| `src/shared/ui/components/Skeleton.tsx` | Crear | Componente skeleton base |

### Dependencies (package.json)

| Paquete | Acción | Descripción |
|---------|--------|-------------|
| `chart.js` | Agregar | Gráficos |
| `react-chartjs-2` | Agregar | Wrapper React para chart.js |
| `jspdf` | Agregar | Generación de PDFs |
| `jspdf-autotable` | Agregar | Tablas en PDFs |
| `animejs` | Agregar | Animaciones |

---

## Interfaces / Contratos

### DTOs del Backend

```rust
// employee.rs
pub struct EmployeeDto {
    pub id: String,
    pub user_id: Option<String>,
    pub document_type: String,
    pub document_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: String,
    pub department: String,
    pub contract_type: String,
    pub base_salary: f64,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub account_type: Option<String>,
    pub cci: Option<String>,
    pub afp: Option<String>,
    pub hire_date: String,
    pub termination_date: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct CreateEmployeeRequest { /* ... */ }
pub struct UpdateEmployeeRequest { /* ... */ }

// payroll.rs
pub struct PayrollRunDto {
    pub id: String,
    pub period_start: String,
    pub period_end: String,
    pub status: String,
    pub total_gross: f64,
    pub total_deductions: f64,
    pub total_net: f64,
    pub created_at: String,
    pub created_by: String,
}

pub struct PayrollEntryDto {
    pub id: String,
    pub payroll_run_id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub base_salary: f64,
    pub hours_worked: f64,
    pub overtime_hours: f64,
    pub overtime_amount: f64,
    pub bonuses: f64,
    pub commissions: f64,
    pub mobility: f64,
    pub food: f64,
    pub other_income: f64,
    pub afp_deduction: f64,
    pub onp_deduction: f64,
    pub essalud: f64,
    pub itf: f64,
    pub other_deductions: f64,
    pub gross_income: f64,
    pub net_income: f64,
    pub status: String,
}

// accounting.rs
pub struct AccountingEntryDto {
    pub id: String,
    pub date: String,
    pub reference: String,
    pub description: String,
    pub debit_account: String,
    pub credit_account: String,
    pub amount: f64,
    pub entry_type: String,
    pub related_id: Option<String>,
    pub related_type: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

pub struct AccountCategoryDto {
    pub id: String,
    pub code: String,
    pub name: String,
    pub category_type: String,
    pub parent_id: Option<String>,
    pub balance: f64,
    pub active: bool,
}

// invoice.rs
pub struct InvoiceDto {
    pub id: String,
    pub series: String,
    pub number: String,
    pub client_name: String,
    pub client_ruc: String,
    pub client_address: Option<String>,
    pub emission_date: String,
    pub due_date: String,
    pub subtotal: f64,
    pub igv: f64,
    pub total: f64,
    pub status: String,
    pub payment_method: Option<String>,
    pub paid_date: Option<String>,
    pub created_at: String,
}
```

### Tipos TypeScript del Frontend

```typescript
// types/accounting.ts
export interface Employee {
  id: string;
  userId?: string;
  documentType: 'DNI' | 'CE' | 'RUC' | 'PASSPORT';
  documentNumber: string;
  firstName: string;
  lastName: string;
  email: string;
  phone?: string;
  address?: string;
  position: string;
  department: string;
  contractType: 'fixed' | 'indefinite' | 'hourly' | 'services';
  baseSalary: number;
  bankName?: string;
  bankAccount?: string;
  accountType?: 'savings' | 'checking';
  cci?: string;
  afp?: 'prima' | 'habitat' | 'integra' | 'profuturo' | 'onp';
  hireDate: string;
  terminationDate?: string;
  status: 'active' | 'inactive' | 'terminated';
  createdAt: string;
  updatedAt: string;
}

export interface PayrollRun {
  id: string;
  periodStart: string;
  periodEnd: string;
  status: 'draft' | 'calculated' | 'confirmed' | 'cancelled';
  totalGross: number;
  totalDeductions: number;
  totalNet: number;
  createdAt: string;
  createdBy: string;
}

export interface PayrollEntry {
  id: string;
  payrollRunId: string;
  employeeId: string;
  employeeName: string;
  baseSalary: number;
  hoursWorked: number;
  overtimeHours: number;
  overtimeAmount: number;
  bonuses: number;
  commissions: number;
  mobility: number;
  food: number;
  otherIncome: number;
  afpDeduction: number;
  onpDeduction: number;
  essalud: number;
  itf: number;
  otherDeductions: number;
  grossIncome: number;
  netIncome: number;
  status: 'calculated' | 'paid' | 'cancelled';
}

export interface AccountingEntry {
  id: string;
  date: string;
  reference: string;
  description: string;
  debitAccount: string;
  creditAccount: string;
  amount: number;
  entryType: 'manual' | 'automatic' | 'adjustment';
  relatedId?: string;
  relatedType?: string;
  createdAt: string;
  createdBy: string;
}

export interface AccountCategory {
  id: string;
  code: string;
  name: string;
  categoryType: 'asset' | 'liability' | 'equity' | 'expense' | 'cost' | 'income';
  parentId?: string;
  balance: number;
  active: boolean;
}

export interface Invoice {
  id: string;
  series: string;
  number: string;
  clientName: string;
  clientRuc: string;
  clientAddress?: string;
  emissionDate: string;
  dueDate: string;
  subtotal: number;
  igv: number;
  total: number;
  status: 'pending' | 'paid' | 'overdue' | 'cancelled';
  paymentMethod?: 'cash' | 'transfer' | 'card';
  paidDate?: string;
  createdAt: string;
}
```

---

## Estrategia de Testing

| Capa | Qué Testear | Enfoque |
|------|-------------|---------|
| Unit (Rust) | Cálculos de nómina (AFP, Essalud, ITF), validadores | Tests unitarios en `src-tauri/src/` con `cargo test` |
| Unit (React) | Hooks useEmployees, usePayroll, componentes de formulario | Vitest con mocking de invoke |
| Integration | CRUD completo de empleados, cálculo de nómina | Tests de integración en Rust |
| E2E | Crear empleado, ejecutar nómina, generar reporte | Playwright en `tests/e2e/` |

### Casos de Prueba Prioritarios (Rust)

```rust
// payroll calculation tests
#[test]
fn test_afp_deduction_prima() {
    let salary = 3000.0;
    let deduction = calculate_afp(salary, "prima");
    assert!((deduction - 337.50).abs() < 0.01); // 11.25%
}

#[test]
fn test_essalud_calculation() {
    let salary = 3000.0;
    let essalud = calculate_essalud(salary);
    assert!((essalud - 270.0).abs() < 0.01); // 9%
}

#[test]
fn test_itf_applies_above_threshold() {
    let deposit = 4000.0;
    let itf = calculate_itf(deposit);
    assert!((itf - 20.0).abs() < 0.01); // 0.5%
}

#[test]
fn test_itf_not_applies_below_threshold() {
    let deposit = 3000.0;
    let itf = calculate_itf(deposit);
    assert!(itf == 0.0);
}
```

---

## Migración / Rollout

### Migración de Datos

No se requiere migración de datos existente. Las nuevas tablas se crean en una nueva migración (`010_accounting_schema.sql`).

### Plan de Implementación por Fases

1. **Fase 1**: Entidades, puertos y repositorios SQLite
2. **Fase 2**: Use cases con lógica de negocio (cálculo de nómina)
3. **Fase 3**: Comandos Tauri
4. **Fase 4**: Frontend - CRUD de empleados
5. **Fase 5**: Frontend - Nómina y asientos contables
6. **Fase 6**: Frontend - Reportes y gráficos
7. **Fase 7**: Frontend - Facturación

### Feature Flags

No se requieren feature flags. El módulo se habilita completamente al completar todas las fases.

---

## Preguntas Abiertas

- [ ] ¿El proyecto necesita integración con sistema de backup externo o es suficiente el export/import SQLite?
- [ ] ¿Se requiere integración con proveedor de email para enviar recibos por correo?
- [ ] ¿El plan de cuentas debe ser configurable o usamos el estándar peruano por defecto?
- [ ] ¿Para facturación electrónica (Sunat) se需要一个 implementación futura o solo PDFs locales?