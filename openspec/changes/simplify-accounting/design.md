# Design: Simplify Accounting Module

## Architecture Decision: Simplified Hexagonal Architecture

### What Stays (All Layers)

The hexagonal architecture pattern remains intact across all four layers. What changes is the **scope and complexity** within each layer.

| Layer | What Stays | What Gets Removed |
|-------|-----------|-------------------|
| **Domain** | `accounting.rs` entity with simplified struct | `CategoryType` enum (asset/liability/equity/cost), `EntryType` enum (automatic/adjustment), parent_id hierarchy |
| **Application** | `AccountingService<R: AccountingRepository>` with generic | `get_trial_balance`, `get_financial_balance`, `get_account_tree`, `list_accounts` (category queries), `get_income_statement` (complex version) |
| **Infrastructure** | `SqliteAccountingRepository` with simplified queries | `SqliteAccountCategoryRepository`, `SqliteLiabilityRepository`, `SqliteEquityRepository` |
| **Commands** | `create_entry`, `list_entries`, `get_accounting_summary` | `get_trial_balance`, `get_income_statement`, `list_accounts`, `get_account_tree`, `get_financial_balance`, all commands in `accounting_ext.rs` |

### Simplified Flow

```
Frontend (React 19)
  ↓ invoke("create_entry", { request })
Tauri Command (accounting.rs)
  ↓ AccountingService.create_entry()
Repository (SqliteAccountingRepository)
  ↓ INSERT INTO accounting_entries
SQLite Database
```

No category lookups, no account trees, no trial balance computation. Just CRUD + summary aggregation.

---

## Database Design

### Current Schema (010_accounting_schema.sql)

```sql
-- accounting_entries: double-entry with PUC account references
CREATE TABLE accounting_entries (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    reference TEXT NOT NULL,
    description TEXT NOT NULL,
    debit_account TEXT NOT NULL,        -- FK to account_categories (PUC code)
    credit_account TEXT NOT NULL,       -- FK to account_categories (PUC code)
    amount REAL NOT NULL,
    entry_type TEXT NOT NULL DEFAULT 'manual',  -- manual | automatic | adjustment
    related_id TEXT,
    related_type TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    FOREIGN KEY (debit_account) REFERENCES account_categories(id),
    FOREIGN KEY (credit_account) REFERENCES account_categories(id)
);
```

Also depends on: `account_categories` (80+ PUC accounts), `liabilities`, `equities`, `fixed_assets`, `asset_depreciation`

### Proposed Simplified Schema

**Strategy**: ALTER TABLE to add `type` and `category` columns, then migrate data. Keep the existing table structure but make PUC references nullable.

```sql
-- Migration 016: Simplify accounting_entries
-- Add simplified columns alongside existing PUC columns

ALTER TABLE accounting_entries ADD COLUMN type TEXT CHECK(type IN ('income', 'expense'));
ALTER TABLE accounting_entries ADD COLUMN category TEXT CHECK(category IN (
    'tuition', 'other_income',           -- income categories
    'rent', 'salaries', 'utilities', 'other_expense'  -- expense categories
));

-- Make PUC columns nullable (they were NOT NULL before)
-- SQLite doesn't support ALTER COLUMN, so we need to recreate the table

-- Step 1: Create new simplified table
CREATE TABLE accounting_entries_v2 (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    reference TEXT NOT NULL,
    description TEXT NOT NULL,
    amount REAL NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('income', 'expense')),
    category TEXT NOT NULL CHECK(category IN (
        'tuition', 'other_income',
        'rent', 'salaries', 'utilities', 'other_expense'
    )),
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL
);

-- Step 2: Migrate existing data (map PUC codes to categories)
INSERT INTO accounting_entries_v2 (id, date, reference, description, amount, type, category, created_at, created_by)
SELECT
    id, date, reference, description, amount,
    CASE
        WHEN credit_account LIKE '6%' THEN 'income'
        WHEN debit_account LIKE '4%' OR debit_account LIKE '5%' THEN 'expense'
        ELSE 'expense'  -- default
    END as type,
    CASE
        WHEN credit_account IN ('6110', '6115') THEN 'tuition'
        WHEN credit_account LIKE '6%' THEN 'other_income'
        WHEN debit_account IN ('4105', '4110', '4115', '4120', '4130', '4135', '4140', '4145') THEN 'salaries'
        WHEN debit_account IN ('4210') THEN 'rent'
        WHEN debit_account IN ('4215', '4220', '4225', '4230', '4235') THEN 'utilities'
        ELSE 'other_expense'
    END as category,
    created_at, created_by
FROM accounting_entries;

-- Step 3: Drop old table and rename
DROP TABLE accounting_entries;
ALTER TABLE accounting_entries_v2 RENAME TO accounting_entries;

-- Recreate indexes
CREATE INDEX idx_accounting_entries_date ON accounting_entries(date);
CREATE INDEX idx_accounting_entries_type ON accounting_entries(type);
CREATE INDEX idx_accounting_entries_category ON accounting_entries(category);

-- Step 4: Remove unused tables and migrations
-- (handled by separate cleanup migration)
```

### Migrations to Remove

| Migration | Content | Action |
|-----------|---------|--------|
| `011_accounting_seed.sql` | 80+ PUC account inserts | DELETE file |
| `012_liabilities_equity_schema.sql` | liabilities, equities tables | DELETE file |
| `013_fixed_assets_schema.sql` | fixed_assets, asset_depreciation tables | DELETE file |
| `014_fixed_assets_accounts.sql` | More PUC accounts (15xx, 16xx) | DELETE file |
| `015_pasivos_accounts.sql` | More PUC accounts (21xx, 22xx) | DELETE file |

### Migration Strategy

Since SQLite doesn't support `ALTER COLUMN`, we use the **table recreation pattern**:
1. Create new table with simplified schema
2. Migrate data with PUC-to-category mapping
3. Drop old table, rename new
4. Recreate indexes

Existing data is preserved but PUC account references are converted to simplified categories. Entries that don't map cleanly default to `other_expense`.

---

## Backend Architecture

### Simplified Domain Entity

```rust
// src-tauri/src/domain/entities/accounting.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    Income,
    Expense,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IncomeCategory {
    Tuition,      // Mensualidades, matrículas
    OtherIncome,  // Cursos especiales, arrendamientos, otros
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpenseCategory {
    Rent,          // Arrendamiento
    Salaries,      // Sueldos y nómina
    Utilities,     // Servicios, agua, luz, teléfono
    OtherExpense,  // Mantenimiento, otros
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingEntry {
    pub id: String,
    pub date: String,
    pub reference: String,
    pub description: String,
    pub amount: f64,
    pub entry_type: EntryType,
    pub category: String,  // "tuition" | "other_income" | "rent" | "salaries" | "utilities" | "other_expense"
    pub created_at: String,
    pub created_by: String,
}
```

### Simplified DTOs

```rust
// src-tauri/src/application/dto/accounting.rs

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub entry_type: String,  // "income" | "expense"
    pub category: String,    // enum value
    pub reference: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccountingEntryDto {
    pub id: String,
    pub date: String,
    pub reference: String,
    pub description: String,
    pub amount: f64,
    pub entry_type: String,
    pub category: String,
    pub created_at: String,
    pub created_by: String,
}

#[derive(Debug, Serialize)]
pub struct AccountingSummary {
    pub total_income: f64,
    pub total_expenses: f64,
    pub net_balance: f64,
    pub entry_count: i64,
    pub recent_entries: Vec<AccountingEntryDto>,
    pub monthly_data: Vec<MonthlyDataPoint>,
    pub expenses_by_category: Vec<CategoryBreakdown>,
    pub income_by_category: Vec<CategoryBreakdown>,
}

#[derive(Debug, Serialize)]
pub struct MonthlyDataPoint {
    pub month: String,
    pub income: f64,
    pub expenses: f64,
}

#[derive(Debug, Serialize)]
pub struct CategoryBreakdown {
    pub category: String,
    pub amount: f64,
}
```

### Simplified Repository

```rust
// src-tauri/src/infrastructure/repositories/sqlite/accounting.rs

pub trait AccountingRepository {
    fn create(&self, entry: &AccountingEntry) -> Result<AccountingEntry, String>;
    fn list(&self, date_from: Option<&str>, date_to: Option<&str>, entry_type: Option<&str>) -> Result<Vec<AccountingEntry>, String>;
    fn get(&self, id: &str) -> Result<Option<AccountingEntry>, String>;
    fn summary(&self) -> Result<AccountingSummary, String>;
}
```

SQL queries become simple:
- `INSERT INTO accounting_entries (id, date, reference, description, amount, type, category, created_at, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`
- `SELECT * FROM accounting_entries WHERE date BETWEEN ? AND ? AND type = ? ORDER BY date DESC`
- `SELECT type, category, SUM(amount), COUNT(*) FROM accounting_entries GROUP BY type, category`

### Simplified Commands

```rust
// src-tauri/src/commands/accounting.rs

#[command]
pub fn create_entry(
    state: State<AccountingServiceState>,
    request: CreateEntryRequest,
    created_by: String,
) -> Result<AccountingEntryDto, String>

#[command]
pub fn list_entries(
    state: State<AccountingServiceState>,
    date_from: Option<String>,
    date_to: Option<String>,
    entry_type: Option<String>,
) -> Result<Vec<AccountingEntryDto>, String>

#[command]
pub fn get_accounting_summary(
    state: State<AccountingServiceState>,
) -> Result<AccountingSummary, String>
```

### Categories: Enum in Code, String in DB

Categories are defined as Rust enums for type safety but stored as strings in SQLite:

```rust
pub const INCOME_CATEGORIES: &[&str] = &["tuition", "other_income"];
pub const EXPENSE_CATEGORIES: &[&str] = &["rent", "salaries", "utilities", "other_expense"];
```

No foreign key to `account_categories` table (which gets deleted).

---

## Frontend Architecture

### Component Tree

```
AccountingPage
├── PeriodSelector (date range)
├── StatsCards
│   ├── StatCard ("Total Ingresos", totalIncome)
│   ├── StatCard ("Total Gastos", totalExpenses)
│   └── StatCard ("Balance Neto", netBalance)
├── ChartsGrid
│   ├── IncomeExpensesChart (Bar - recharts)
│   ├── MonthlyTrendChart (Line - recharts)
│   ├── ExpenseBreakdownChart (Pie - recharts)
│   └── ProfitMarginChart (custom gauge)
├── RecentEntriesTable
│   └── EntryRow (date, reference, description, amount, category badge)
└── Modals
    ├── IncomeForm (react-hook-form + zod)
    └── ExpenseForm (react-hook-form + zod)
```

### State Management: Custom Hook

```typescript
// src/features/accounting/hooks/useAccounting.ts

export function useAccounting() {
  // State
  const [entries, setEntries] = useState<AccountingEntry[]>([]);
  const [summary, setSummary] = useState<AccountingSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Actions
  const createEntry = async (request: CreateEntryRequest) => { ... };
  const listEntries = async (filters?: EntryFilters) => { ... };
  const getSummary = async () => { ... };

  return { entries, summary, loading, error, createEntry, listEntries, getSummary };
}
```

**Removed from hook**: `listAccounts`, `getAccountTree`, `getTrialBalance`, `getIncomeStatement`, `getFinancialBalance`, `createLiability`, `listLiabilities`, `payLiability`, `createEquity`, `listEquities`, `createFixedAsset`

### Form Handling

```typescript
// IncomeForm with react-hook-form + zod
const incomeSchema = z.object({
  date: z.string().min(1, "Fecha requerida"),
  description: z.string().min(1, "Descripción requerida"),
  amount: z.number().positive("Monto debe ser positivo"),
  category: z.enum(["tuition", "other_income"]),
  reference: z.string().optional(),
});

// ExpenseForm with react-hook-form + zod
const expenseSchema = z.object({
  date: z.string().min(1, "Fecha requerida"),
  description: z.string().min(1, "Descripción requerida"),
  amount: z.number().positive("Monto debe ser positivo"),
  category: z.enum(["rent", "salaries", "utilities", "other_expense"]),
  reference: z.string().optional(),
});
```

### Chart Library: recharts (NOT chart.js)

The original design chose chart.js, but the existing `DashboardCharts.tsx` already uses `react-chartjs-2`. However, the proposal says "recharts" for the simplified version. Since we're rewriting the charts anyway, we'll switch to **recharts** for consistency with the React ecosystem and simpler API.

**Decision**: Use recharts. Remove chart.js and react-chartjs-2 dependencies.

### Type Definitions

```typescript
// src/features/accounting/types/accounting.ts

export const INCOME_CATEGORIES = {
  TUITION: "tuition",
  OTHER_INCOME: "other_income",
} as const;

export type IncomeCategory = typeof INCOME_CATEGORIES[keyof typeof INCOME_CATEGORIES];

export const EXPENSE_CATEGORIES = {
  RENT: "rent",
  SALARIES: "salaries",
  UTILITIES: "utilities",
  OTHER_EXPENSE: "other_expense",
} as const;

export type ExpenseCategory = typeof EXPENSE_CATEGORIES[keyof typeof EXPENSE_CATEGORIES];

export type EntryType = "income" | "expense";

export interface CreateEntryRequest {
  date: string;
  description: string;
  amount: number;
  entry_type: EntryType;
  category: IncomeCategory | ExpenseCategory;
  reference?: string;
}

export interface AccountingEntry {
  id: string;
  date: string;
  reference: string;
  description: string;
  amount: number;
  entry_type: EntryType;
  category: string;
  created_at: string;
  created_by: string;
}

export interface MonthlyDataPoint {
  month: string;
  income: number;
  expenses: number;
}

export interface CategoryBreakdown {
  category: string;
  amount: number;
}

export interface AccountingSummary {
  total_income: number;
  total_expenses: number;
  net_balance: number;
  entry_count: number;
  recent_entries: AccountingEntry[];
  monthly_data: MonthlyDataPoint[];
  expenses_by_category: CategoryBreakdown[];
  income_by_category: CategoryBreakdown[];
}

export interface EntryFilters {
  date_from?: string;
  date_to?: string;
  entry_type?: EntryType;
}
```

**Removed types**: `AccountCategory`, `AccountCategoryTreeNode`, `TrialBalance`, `TrialBalanceAccount`, `IncomeStatement`, `FinancialBalance`, `AccountBalance`, `Liability`, `CreateLiabilityRequest`, `Equity`, `CreateEquityRequest`, `FixedAsset`, `CreateFixedAssetRequest`, `LiabilityType`, `EquityType`, `CategoryType`

---

## File-by-File Changes

### Backend - Files to DELETE

| File | Reason |
|------|--------|
| `src-tauri/migrations/011_accounting_seed.sql` | PUC chart of accounts seed |
| `src-tauri/migrations/012_liabilities_equity_schema.sql` | liabilities, equities tables |
| `src-tauri/migrations/013_fixed_assets_schema.sql` | fixed_assets, asset_depreciation tables |
| `src-tauri/migrations/014_fixed_assets_accounts.sql` | More PUC accounts |
| `src-tauri/migrations/015_pasivos_accounts.sql` | More PUC accounts |
| `src-tauri/src/commands/accounting_ext.rs` | Liability, equity, fixed asset commands |
| `src-tauri/src/domain/entities/liability.rs` | Liability entity |
| `src-tauri/src/domain/entities/equity.rs` | Equity entity |
| `src-tauri/src/domain/entities/fixed_asset.rs` | Fixed asset entity |
| `src-tauri/src/application/use_cases/liability.rs` | Liability service |
| `src-tauri/src/application/use_cases/equity.rs` | Equity service |
| `src-tauri/src/infrastructure/repositories/sqlite/liability.rs` | SQLite liability repo |
| `src-tauri/src/infrastructure/repositories/sqlite/equity.rs` | SQLite equity repo |

### Backend - Files to MODIFY

| File | Change |
|------|--------|
| `src-tauri/src/domain/entities/accounting.rs` | Simplify: remove PUC enums, add EntryType/IncomeCategory/ExpenseCategory |
| `src-tauri/src/domain/entities/mod.rs` | Remove liability, equity, fixed_asset exports |
| `src-tauri/src/application/dto/accounting.rs` | Simplify: remove TrialBalance, FinancialBalance, IncomeStatement DTOs |
| `src-tauri/src/application/dto/mod.rs` | Remove liability, equity, fixed_asset DTO exports |
| `src-tauri/src/application/use_cases/accounting.rs` | Simplify: remove trial_balance, financial_balance, account_tree, list_accounts |
| `src-tauri/src/application/use_cases/mod.rs` | Remove liability, equity service exports |
| `src-tauri/src/infrastructure/repositories/sqlite/accounting.rs` | Simplify: remove account_category queries, simplify entry queries |
| `src-tauri/src/infrastructure/repositories/mod.rs` | Remove liability, equity repo exports |
| `src-tauri/src/commands/accounting.rs` | Remove trial_balance, income_statement, financial_balance, list_accounts, get_account_tree commands |
| `src-tauri/src/commands/mod.rs` | Remove accounting_ext module, register simplified commands |
| `src-tauri/src/lib.rs` | Remove accounting_ext commands from handler, add migration 016 |

### Backend - Files to CREATE

| File | Purpose |
|------|---------|
| `src-tauri/migrations/016_simplify_accounting.sql` | Table recreation migration |

### Frontend - Files to DELETE

| File | Reason |
|------|--------|
| `src/features/accounting/components/LiabilityForm.tsx` | Liability form (out of scope) |
| `src/features/accounting/components/EquityForm.tsx` | Equity form (out of scope) |
| `src/features/accounting/components/FixedAssetForm.tsx` | Fixed asset form (out of scope) |
| `src/features/accounting/components/PDFGenerator.ts` | PDF export (out of scope) |
| `src/features/accounting/routes/ReportsPage.tsx` | Reports page (out of scope) |
| `src/features/accounting/routes/EmployeesPage.tsx` | Employees page (out of scope) |
| `src/features/accounting/routes/PayrollPage.tsx` | Payroll page (out of scope) |
| `src/features/accounting/hooks/useEmployees.ts` | Employees hook (out of scope) |
| `src/features/accounting/hooks/usePayroll.ts` | Payroll hook (out of scope) |
| `src/features/accounting/types/employees.ts` | Employee types (out of scope) |
| `src/features/accounting/types/payroll.ts` | Payroll types (out of scope) |
| `src/features/accounting/lib/animations.ts` | Animation utilities (keep if used elsewhere, else remove) |

### Frontend - Files to MODIFY

| File | Change |
|------|--------|
| `src/features/accounting/types/accounting.ts` | Complete rewrite: simplified types only |
| `src/features/accounting/types/index.ts` | Remove employees, payroll exports |
| `src/features/accounting/hooks/useAccounting.ts` | Simplify: remove liability, equity, fixed asset, trial balance logic |
| `src/features/accounting/hooks/index.ts` | Remove employees, payroll exports |
| `src/features/accounting/components/IncomeForm.tsx` | Simplify: remove PUC account mapping, use category enum |
| `src/features/accounting/components/ExpenseForm.tsx` | Simplify: remove PUC account mapping, use category enum |
| `src/features/accounting/components/DashboardCharts.tsx` | Rewrite: switch from chart.js to recharts |
| `src/features/accounting/components/AccountingTable.tsx` | Simplify: remove PUC columns, show category badges |
| `src/features/accounting/components/index.ts` | Remove LiabilityForm, EquityForm, FixedAssetForm, PDFGenerator exports |
| `src/features/accounting/routes/AccountingPage.tsx` | Simplify: remove liability/equity/fixed_asset modals and lists |
| `src/features/accounting/routes/index.ts` | Remove ReportsPage, EmployeesPage, PayrollPage exports |
| `src/features/accounting/index.ts` | Update exports |
| `src/app/layouts/MainLayout.tsx` | Remove "Empleados", "Nómina" navigation entries, simplify page type |

### Frontend - Files to CREATE

| File | Purpose |
|------|---------|
| `src/features/accounting/components/IncomeForm.tsx` | (rewrite) Simplified with react-hook-form + zod |
| `src/features/accounting/components/ExpenseForm.tsx` | (rewrite) Simplified with react-hook-form + zod |

### Dependencies

| Package | Action | Reason |
|---------|--------|--------|
| `chart.js` | REMOVE | Replaced by recharts |
| `react-chartjs-2` | REMOVE | Replaced by recharts |
| `recharts` | ADD | Chart library for dashboard |
| `react-hook-form` | ADD | Form state management |
| `zod` | ADD | Schema validation |
| `jspdf` | REMOVE | PDF export removed |
| `jspdf-autotable` | REMOVE | PDF export removed |

---

## Implementation Order

### Phase 1: Database Migration (Zero Downtime)

1. Create `016_simplify_accounting.sql` migration
2. Test migration on fresh database
3. Test migration on existing database with data

### Phase 2: Backend Simplification

1. Simplify `domain/entities/accounting.rs` (new enums)
2. Simplify `application/dto/accounting.rs` (remove complex DTOs)
3. Simplify `application/use_cases/accounting.rs` (remove complex queries)
4. Simplify `infrastructure/repositories/sqlite/accounting.rs` (new queries)
5. Simplify `commands/accounting.rs` (remove commands)
6. Delete `commands/accounting_ext.rs`
7. Update `commands/mod.rs` and `lib.rs`
8. Run `cargo check` to verify compilation

### Phase 3: Frontend Type Cleanup

1. Rewrite `types/accounting.ts` (simplified types)
2. Update `types/index.ts`
3. Run `bunx tsc --noEmit` to verify types

### Phase 4: Frontend Hook Simplification

1. Rewrite `hooks/useAccounting.ts` (simplified hook)
2. Update `hooks/index.ts`
3. Run `bunx tsc --noEmit`

### Phase 5: Frontend Component Simplification

1. Rewrite `components/IncomeForm.tsx` (react-hook-form + zod)
2. Rewrite `components/ExpenseForm.tsx` (react-hook-form + zod)
3. Rewrite `components/DashboardCharts.tsx` (recharts)
4. Simplify `components/AccountingTable.tsx`
5. Update `components/index.ts`
6. Run `bunx tsc --noEmit`

### Phase 6: Frontend Page & Navigation

1. Simplify `routes/AccountingPage.tsx`
2. Update `routes/index.ts`
3. Update `app/layouts/MainLayout.tsx` (remove nav entries)
4. Delete unused route files
5. Delete unused hook files
6. Delete unused type files
7. Run `bunx tsc --noEmit && bun run build`

### Phase 7: Cleanup

1. Delete migration files 011-015
2. Delete backend entity/use_case/repo files for liability, equity, fixed_asset
3. Remove chart.js, jspdf dependencies
4. Add recharts, react-hook-form, zod dependencies
5. Final `bun run build` verification

---

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Existing accounting entries have PUC references that don't map to categories | Medium | Default unmapped entries to `other_expense`; log warnings during migration |
| Users lose visibility into historical PUC-based data | Low | Migration preserves data; PUC codes are preserved in the mapping logic |
| chart.js to recharts migration breaks existing chart styling | Medium | Keep same color palette; recharts has similar API |
| Removing employees/payroll pages breaks navigation | Low | These pages are already separate modules; remove nav entries only |

---

## Success Criteria

- [ ] Single "Contabilidad" navigation entry (no Empleados, Nómina, Reportes)
- [ ] IncomeForm with categories: Tuition, Other Income
- [ ] ExpenseForm with categories: Rent, Salaries, Utilities, Other Expense
- [ ] Dashboard showing: Total Income, Total Expenses, Net Balance
- [ ] Dashboard charts: Income vs Expenses (bar), Monthly Trend (line), Expense Breakdown (pie)
- [ ] No references to PUC, trial balance, financial balance, fixed assets, liabilities, or equity
- [ ] Backend reduced to 3 commands: create_entry, list_entries, get_accounting_summary
- [ ] `bunx tsc --noEmit` passes
- [ ] `bun run build` succeeds
- [ ] `cargo check` passes
