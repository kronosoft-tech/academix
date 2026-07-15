# Delta for Accounting Module Simplification

## ADDED Requirements

### Requirement: Income Entry CRUD

The system SHALL allow users to create, list, and delete income entries. Each income entry represents money received by the academy.

**Fields:**
- `id` (TEXT, UUID, auto-generated)
- `date` (TEXT, ISO 8601 date, required)
- `category` (TEXT, enum: "tuition" | "other", required)
- `description` (TEXT, required, max 500 chars)
- `amount` (REAL, required, > 0)
- `created_at` (TEXT, ISO 8601 datetime, auto)

#### Scenario: Create a tuition income entry

- GIVEN the user is on the Accounting page
- WHEN the user fills the Income form with date="2026-07-14", category="tuition", description="Monthly fee - Student A", amount=500
- AND clicks "Guardar"
- THEN a new income entry is created with the provided fields
- AND the summary totals are refreshed
- AND the entry appears in the entries table

#### Scenario: Create an income entry with invalid amount

- GIVEN the user is on the Accounting page
- WHEN the user fills the Income form with amount=0 or amount=-100
- AND clicks "Guardar"
- THEN the form shows a validation error
- AND no entry is created

#### Scenario: Delete an income entry

- GIVEN the entries table shows at least one income entry
- WHEN the user clicks the delete button on an income entry
- AND confirms the deletion
- THEN the entry is removed from the database
- AND the summary totals are refreshed

### Requirement: Expense Entry CRUD

The system SHALL allow users to create, list, and delete expense entries. Each expense entry represents money spent by the academy.

**Fields:**
- `id` (TEXT, UUID, auto-generated)
- `date` (TEXT, ISO 8601 date, required)
- `category` (TEXT, enum: "rent" | "salaries" | "utilities" | "other", required)
- `description` (TEXT, required, max 500 chars)
- `amount` (REAL, required, > 0)
- `created_at` (TEXT, ISO 8601 datetime, auto)

#### Scenario: Create a rent expense entry

- GIVEN the user is on the Accounting page
- WHEN the user fills the Expense form with date="2026-07-14", category="rent", description="July rent", amount=1200
- AND clicks "Guardar"
- THEN a new expense entry is created with the provided fields
- AND the summary totals are refreshed
- AND the entry appears in the entries table

#### Scenario: Create an expense entry with missing required fields

- GIVEN the user is on the Accounting page
- WHEN the user leaves the description field empty
- AND clicks "Guardar"
- THEN the form shows a validation error for the description field
- AND no entry is created

#### Scenario: Delete an expense entry

- GIVEN the entries table shows at least one expense entry
- WHEN the user clicks the delete button on an expense entry
- AND confirms the deletion
- THEN the entry is removed from the database
- AND the summary totals are refreshed

### Requirement: Date Range Filtering

The system SHALL filter entries and summary statistics by a user-selected date range. The default range SHALL be the current month (first day to today).

#### Scenario: Filter entries by date range

- GIVEN the user is on the Accounting page
- WHEN the user sets start date to "2026-07-01" and end date to "2026-07-31"
- THEN the entries table shows only entries within that range
- AND the summary cards show totals for that range only
- AND the charts reflect data for that range only

#### Scenario: Reset date range to current month

- GIVEN the user has changed the date range to a custom period
- WHEN the page loads or the user clicks a "This Month" reset button
- THEN the date range resets to the first day of the current month through today

### Requirement: Dashboard Summary Cards

The system SHALL display three summary cards at the top of the Accounting page:

1. **Total Income** — sum of all income entries in the selected date range, formatted as `S/ {amount}`
2. **Total Expenses** — sum of all expense entries in the selected date range, formatted as `S/ {amount}`
3. **Net Balance** — Total Income minus Total Expenses, formatted as `S/ {amount}`, positive in green, negative in red

#### Scenario: Summary cards reflect date range

- GIVEN there are income entries totaling S/ 5000 and expense entries totaling S/ 3000 in July 2026
- WHEN the user selects the date range July 2026
- THEN the Total Income card shows "S/ 5,000.00"
- AND the Total Expenses card shows "S/ 3,000.00"
- AND the Net Balance card shows "S/ 2,000.00" in green

#### Scenario: Summary cards with no data

- GIVEN the user selects a date range with no entries
- WHEN the page loads
- THEN all three cards show "S/ 0.00"

### Requirement: Dashboard Charts

The system SHALL display three charts on the Accounting page:

1. **Income vs Expenses Bar Chart** — horizontal or vertical bar comparing total income and total expenses for the selected period
2. **Monthly Trend Line Chart** — line chart showing income and expenses over the last 6 months (or available data)
3. **Expense Breakdown Doughnut Chart** — doughnut chart showing expense distribution by category (Rent, Salaries, Utilities, Other)

#### Scenario: Charts update on date range change

- GIVEN the user changes the date range filter
- WHEN the new range is applied
- THEN all three charts re-render with data from the new range

#### Scenario: Expense breakdown with single category

- GIVEN all expenses in the period are in the "rent" category
- WHEN the doughnut chart renders
- THEN it shows a single solid circle labeled "Rent 100%"

### Requirement: Entries Table

The system SHALL display a sortable table of accounting entries (both income and expense) below the charts. The table SHALL show columns: Date, Type (Income/Expense), Category, Description, Amount, and a Delete action.

Entries SHALL be sorted by date descending (newest first) by default.

#### Scenario: Table shows mixed income and expense entries

- GIVEN there are 3 income entries and 2 expense entries in the selected period
- WHEN the page loads
- THEN the table shows 5 rows sorted by date descending
- AND each row shows the correct type badge (green for Income, red for Expense)

#### Scenario: Empty table state

- GIVEN the selected date range has no entries
- WHEN the page loads
- THEN the table shows a "No hay asientos en este período" message

---

## MODIFIED Requirements

### Requirement: Accounting Page Layout

The AccountingPage layout SHALL be simplified to a single-page view with: Header (title + "Nuevo Ingreso" / "Nuevo Gasto" buttons), Date Range Selector, Summary Cards row, Charts grid (2 columns), and Entries Table. All modal forms (Liability, Equity, FixedAsset) SHALL be removed.

(Previously: Page included modals for Income, Expense, Liability, Equity, and FixedAsset; tables for Liabilities and Equities; PDF export button; payment sync button.)

#### Scenario: Simplified page renders correctly

- GIVEN the user navigates to the Accounting page
- WHEN the page loads
- THEN only Income and Expense forms are accessible via buttons
- AND no Liability, Equity, or FixedAsset buttons are visible
- AND no PDF export button is visible
- AND no payment sync button is visible

### Requirement: Income Form Fields

The IncomeForm SHALL have three fields: Date (date picker, default today), Category (select: Tuition, Other), Description (text input), Amount (number input, S/ prefix). The form SHALL use simple validation (all fields required, amount > 0).

(Previously: IncomeForm had complex PUC account code mapping with account selection dropdowns.)

#### Scenario: Income form has simplified fields

- GIVEN the user clicks "Nuevo Ingreso"
- WHEN the form modal opens
- THEN it shows Date, Category (Tuition/Other), Description, and Amount fields
- AND no account code or PUC-related fields are shown

### Requirement: Expense Form Fields

The ExpenseForm SHALL have three fields: Date (date picker, default today), Category (select: Rent, Salaries, Utilities, Other), Description (text input), Amount (number input, S/ prefix). The form SHALL use simple validation (all fields required, amount > 0).

(Previously: ExpenseForm had complex PUC account code mapping with account selection dropdowns.)

#### Scenario: Expense form has simplified fields

- GIVEN the user clicks "Nuevo Gasto"
- WHEN the form modal opens
- THEN it shows Date, Category (Rent/Salaries/Utilities/Other), Description, and Amount fields
- AND no account code or PUC-related fields are shown

---

## REMOVED Requirements

### Requirement: Double-Entry Bookkeeping

(Reason: Replaced by simple income/expense tracking. The debit_account/credit_account model and chart of accounts hierarchy are unnecessary complexity for this use case.)

The `accounting_entries` table schema SHALL be replaced. The columns `debit_account`, `credit_account`, `entry_type`, `related_id`, `related_type` SHALL be removed. A new simplified `accounting_entries` table SHALL be created with the columns defined in the Income/Expense CRUD requirements above, plus a `type` column (TEXT, enum: "income" | "expense").

### Requirement: Account Categories / Chart of Accounts

(Reason: Replaced by simple category enums. The hierarchical chart of accounts with PUC codes is unnecessary.)

The `account_categories` table SHALL be dropped. Category selection SHALL be limited to the predefined enums for income and expense.

### Requirement: Liability Management (Pasivos)

(Reason: Out of scope for simplified accounting. Liabilities will be tracked separately if needed in the future.)

The following SHALL be removed:
- Frontend: `LiabilityForm.tsx`, `Liability` type, `CreateLiabilityRequest` type, `listLiabilities` hook, liabilities table in AccountingPage
- Backend: `create_liability`, `list_liabilities` commands, liability entity, liability repository
- Database: liability-related columns from `accounting_entries`

### Requirement: Equity Management (Patrimonio)

(Reason: Out of scope for simplified accounting.)

The following SHALL be removed:
- Frontend: `EquityForm.tsx`, `Equity` type, `CreateEquityRequest` type, `listEquities` hook, equities table in AccountingPage
- Backend: `create_equity`, `list_equities` commands, equity entity, equity repository
- Database: equity-related columns from `accounting_entries`

### Requirement: Fixed Asset Management (Activos Fijos)

(Reason: Out of scope for simplified accounting.)

The following SHALL be removed:
- Frontend: `FixedAssetForm.tsx`, `FixedAsset` type, `CreateFixedAssetRequest` type, `createFixedAsset` hook
- Backend: `create_fixed_asset` command, fixed asset entity, fixed asset repository
- Database: `fixed_assets` table and related migrations (013, 014)

### Requirement: Employee Management

(Reason: Out of scope for simplified accounting. Will be handled by a dedicated HR/payroll module if needed.)

The following SHALL be removed:
- Frontend: `EmployeesPage.tsx`, `useEmployees.ts`, `employees.ts` types
- Backend: employee entity, employee repository, employee commands
- Database: `employees` table (migration 010)
- Navigation: "Empleados" sidebar entry

### Requirement: Payroll Management (Nominas)

(Reason: Out of scope for simplified accounting.)

The following SHALL be removed:
- Frontend: `PayrollPage.tsx`, `usePayroll.ts`, `payroll.ts` types
- Backend: payroll run entity, payroll entry entity, payroll repository, payroll commands
- Database: `payroll_runs` and `payroll_entries` tables (migration 010)
- Navigation: "Nominas" sidebar entry

### Requirement: Financial Reports and PDF Export

(Reason: Out of scope for simplified accounting. Reports can be re-added later as a separate feature.)

The following SHALL be removed:
- Frontend: `ReportsPage.tsx`, `PDFGenerator.ts`, `export_financial_balance_pdf` command usage
- Backend: PDF generation commands, financial balance report logic
- Navigation: "Reportes" sidebar entry

### Requirement: Invoice Management

(Reason: Out of scope for simplified accounting.)

The following SHALL be removed from the database:
- `invoices` table (migration 010)
- `invoice_lines` table (migration 010)

### Requirement: Payment Sync to Accounting

(Reason: Simplified module does not need automatic payment synchronization.)

The `syncPaymentsToAccounting` functionality and "Sincronizar Pagos" button SHALL be removed from the Accounting page.

---

## Data Model

### New `accounting_entries` Table Schema

```sql
CREATE TABLE IF NOT EXISTS accounting_entries (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('income', 'expense')),
    category TEXT NOT NULL,
    description TEXT NOT NULL,
    amount REAL NOT NULL CHECK(amount > 0),
    created_at TEXT NOT NULL
);

-- Category constraints per type
-- For type='income': category IN ('tuition', 'other')
-- For type='expense': category IN ('rent', 'salaries', 'utilities', 'other')
-- Enforced at application level, not CHECK constraint (SQLite limitation)

CREATE INDEX IF NOT EXISTS idx_accounting_entries_date ON accounting_entries(date);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_type ON accounting_entries(type);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_category ON accounting_entries(category);
```

### Category Enums

**Income Categories:**
| Value | Display Name |
|-------|-------------|
| `tuition` | Matrícula |
| `other` | Otros |

**Expense Categories:**
| Value | Display Name |
|-------|-------------|
| `rent` | Arriendo |
| `salaries` | Sueldos |
| `utilities` | Servicios |
| `other` | Otros |

### Migration Strategy

A new migration `016_simplify_accounting.sql` SHALL:
1. Create the new simplified `accounting_entries` table
2. Migrate existing income/expense data from the old table (mapping `debit_account` types to new `type` and `category` fields)
3. Drop the old `accounting_entries` table
4. Drop `account_categories`, `employees`, `payroll_runs`, `payroll_entries`, `invoices`, `invoice_lines` tables
5. Migrations 011-015 SHALL be marked as superseded (not deleted, for rollback reference)

---

## API Contract (Tauri Commands)

### `create_income_entry`

```typescript
invoke("create_income_entry", {
  date: string,        // ISO 8601 date
  category: string,    // "tuition" | "other"
  description: string, // max 500 chars
  amount: number       // > 0
}): Promise<AccountingEntry>
```

### `create_expense_entry`

```typescript
invoke("create_expense_entry", {
  date: string,        // ISO 8601 date
  category: string,    // "rent" | "salaries" | "utilities" | "other"
  description: string, // max 500 chars
  amount: number       // > 0
}): Promise<AccountingEntry>
```

### `get_accounting_entries`

```typescript
invoke("get_accounting_entries", {
  date_from?: string,  // ISO 8601 date
  date_to?: string,    // ISO 8601 date
  type?: string        // "income" | "expense" (optional filter)
}): Promise<AccountingEntry[]>
```

### `get_accounting_summary`

```typescript
invoke("get_accounting_summary", {
  date_from?: string,
  date_to?: string
}): Promise<AccountingSummary>
```

**AccountingSummary response:**
```typescript
{
  total_income: number;
  total_expenses: number;
  net_balance: number;
  entry_count: number;
  recent_entries: AccountingEntry[];  // last 10
  monthly_data: { month: string; income: number; expenses: number }[];
  expenses_by_category: { category_name: string; amount: number }[];
  income_by_category: { category_name: string; amount: number }[];
}
```

### `delete_accounting_entry`

```typescript
invoke("delete_accounting_entry", {
  id: string
}): Promise<void>
```

### Removed Commands

The following commands SHALL be deregistered from `lib.rs`:
- `create_liability`, `list_liabilities`, `pay_liability`
- `create_equity`, `list_equities`
- `create_fixed_asset`, `list_fixed_assets`
- `create_employee`, `list_employees`, `update_employee`, `delete_employee`
- `create_payroll_run`, `calculate_payroll`, `confirm_payroll`
- `export_financial_balance_pdf`, `export_trial_balance_pdf`
- `sync_payments_to_accounting`

---

## UI Specifications

### AccountingPage Layout

```
┌─────────────────────────────────────────────────┐
│ Contabilidad                    [+Ingreso] [+Gasto] │
│ Resumen financiero y libros contables           │
├─────────────────────────────────────────────────┤
│ Período: [date_from] - [date_to]                │
├───────────┬───────────┬─────────────────────────┤
│ Ingresos  │ Gastos    │ Balance Neto            │
│ S/ 5,000  │ S/ 3,000  │ S/ 2,000               │
├───────────┴───────────┴─────────────────────────┤
│ [Income vs Expenses Bar]  [Monthly Trend Line]  │
├─────────────────────────────────────────────────┤
│ [Expense Breakdown Doughnut]                    │
├─────────────────────────────────────────────────┤
│ Asientos Recientes                              │
│ Date | Type | Category | Description | Amount | │
│ ...                                             │
└─────────────────────────────────────────────────┘
```

### IncomeForm Modal

```
┌──────────────────────────────────┐
│ Nuevo Ingreso               [✕]  │
├──────────────────────────────────┤
│ Fecha:    [date picker]          │
│ Categoría: [Matrícula | Otros]   │
│ Descripción: [text input]        │
│ Monto:    S/ [number input]      │
├──────────────────────────────────┤
│        [Cancelar] [Guardar]      │
└──────────────────────────────────┘
```

### ExpenseForm Modal

```
┌──────────────────────────────────┐
│ Nuevo Gasto                [✕]   │
├──────────────────────────────────┤
│ Fecha:    [date picker]          │
│ Categoría: [Arriendo|Sueldos|    │
│             Servicios|Otros]     │
│ Descripción: [text input]        │
│ Monto:    S/ [number input]      │
├──────────────────────────────────┤
│        [Cancelar] [Guardar]      │
└──────────────────────────────────┘
```

### Navigation Changes

Remove from sidebar (`MainLayout.tsx`):
- "Empleados" entry
- "Nominas" / "Payroll" entry (if present)
- "Reportes" entry

Keep:
- "Contabilidad" entry (links to simplified AccountingPage)

---

## Acceptance Criteria

1. **Database**: New `accounting_entries` table exists with columns: id, date, type, category, description, amount, created_at
2. **Database**: Old tables (`account_categories`, `employees`, `payroll_runs`, `payroll_entries`, `invoices`, `invoice_lines`) are dropped
3. **Backend**: `create_income_entry` command creates an entry with type="income"
4. **Backend**: `create_expense_entry` command creates an entry with type="expense"
5. **Backend**: `get_accounting_entries` returns entries filtered by date range and optional type
6. **Backend**: `get_accounting_summary` returns correct totals, monthly data, and category breakdowns
7. **Backend**: `delete_accounting_entry` removes an entry by ID
8. **Frontend**: AccountingPage shows 3 summary cards (Income, Expenses, Net Balance)
9. **Frontend**: IncomeForm has Date, Category (Tuition/Other), Description, Amount fields only
10. **Frontend**: ExpenseForm has Date, Category (Rent/Salaries/Utilities/Other), Description, Amount fields only
11. **Frontend**: No Liability, Equity, or FixedAsset forms or buttons are visible
12. **Frontend**: No PDF export or payment sync buttons are visible
13. **Frontend**: Charts render correctly (bar, line, doughnut)
14. **Frontend**: Date range filter updates all sections (cards, charts, table)
15. **Navigation**: Sidebar has no "Empleados", "Nominas", or "Reportes" entries
16. **TypeScript**: `bunx tsc --noEmit` passes with no errors
17. **Build**: `bun run build` succeeds

---

## Coverage

### Happy Paths Covered
- Create income entry ✓
- Create expense entry ✓
- Delete entries ✓
- Filter by date range ✓
- View summary cards ✓
- View charts ✓
- View entries table ✓

### Edge Cases Covered
- Invalid amount (≤ 0) ✓
- Missing required fields ✓
- Empty date range (no data) ✓
- Single category in expense breakdown ✓

### Error States Covered
- Form validation errors ✓
- Empty table state ✓
- Empty summary state ✓

---

## Specs Created

**Change**: simplify-accountability-service

### Specs Written
| Domain | Type | Requirements | Scenarios |
|--------|------|-------------|-----------|
| accounting/entries | New | 3 (Income CRUD, Expense CRUD, Delete) | 9 |
| accounting/dashboard | New | 3 (Summary Cards, Charts, Table) | 6 |
| accounting/filters | New | 1 (Date Range Filtering) | 2 |
| accounting/ui | Modified | 3 (Page Layout, Income Form, Expense Form) | 3 |
| accounting/removals | Removed | 9 (Double-entry, Accounts, Liabilities, Equity, FixedAssets, Employees, Payroll, Reports, Invoices) | 0 |

### Coverage
- Happy paths: 7/7 covered
- Edge cases: 3/3 covered
- Error states: 3/3 covered

### Next Step
Ready for design (sdd-design). If design already exists, ready for tasks (sdd-tasks).
