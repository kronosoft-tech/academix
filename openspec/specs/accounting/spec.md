# Accounting Module Specification

## Overview

Simplified accounting module for Academix. Tracks income and expenses with basic categorization, date filtering, and dashboard visualization.

---

## Requirements

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

### Requirement: Accounting Page Layout

The AccountingPage layout SHALL be a single-page view with: Header (title + "Nuevo Ingreso" / "Nuevo Gasto" buttons), Date Range Selector, Summary Cards row, Charts grid (2 columns), and Entries Table.

#### Scenario: Simplified page renders correctly

- GIVEN the user navigates to the Accounting page
- WHEN the page loads
- THEN only Income and Expense forms are accessible via buttons
- AND the page displays summary cards, charts, and entries table

### Requirement: Income Form Fields

The IncomeForm SHALL have three fields: Date (date picker, default today), Category (select: Tuition, Other), Description (text input), Amount (number input, S/ prefix). The form SHALL use simple validation (all fields required, amount > 0).

#### Scenario: Income form has simplified fields

- GIVEN the user clicks "Nuevo Ingreso"
- WHEN the form modal opens
- THEN it shows Date, Category (Tuition/Other), Description, and Amount fields

### Requirement: Expense Form Fields

The ExpenseForm SHALL have three fields: Date (date picker, default today), Category (select: Rent, Salaries, Utilities, Other), Description (text input), Amount (number input, S/ prefix). The form SHALL use simple validation (all fields required, amount > 0).

#### Scenario: Expense form has simplified fields

- GIVEN the user clicks "Nuevo Gasto"
- WHEN the form modal opens
- THEN it shows Date, Category (Rent/Salaries/Utilities/Other), Description, and Amount fields

---

## Data Model

### `accounting_entries` Table Schema

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

---

## API Contract (Tauri Commands)

### `create_income_entry`

```typescript
invoke("create_income_entry", {
  date: string,
  category: string,
  description: string,
  amount: number
}): Promise<AccountingEntry>
```

### `create_expense_entry`

```typescript
invoke("create_expense_entry", {
  date: string,
  category: string,
  description: string,
  amount: number
}): Promise<AccountingEntry>
```

### `get_accounting_entries`

```typescript
invoke("get_accounting_entries", {
  date_from?: string,
  date_to?: string,
  type?: string
}): Promise<AccountingEntry[]>
```

### `get_accounting_summary`

```typescript
invoke("get_accounting_summary", {
  date_from?: string,
  date_to?: string
}): Promise<AccountingSummary>
```

### `delete_accounting_entry`

```typescript
invoke("delete_accounting_entry", {
  id: string
}): Promise<void>
```

---

## Removed Features (Historical)

The following features were removed in the simplify-accountability-service change:

- Double-entry bookkeeping (debit/credit accounts)
- Account Categories / Chart of Accounts (PUC codes)
- Liability Management (Pasivos)
- Equity Management (Patrimonio)
- Fixed Asset Management (Activos Fijos)
- Employee Management
- Payroll Management (Nominas)
- Financial Reports and PDF Export
- Invoice Management
- Payment Sync to Accounting
