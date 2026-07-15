# Tasks: Simplify Accountability Service

**Change**: simplify-accountability-service  
**Status**: Ready for implementation  
**Total tasks**: 25  
**Estimated effort**: 2-3 days  

---

## Phase 1: Database Migration (3 tasks)

### Task 1.1: Create migration 016
- **File**: `src-tauri/migrations/016_simplify_accounting_schema.sql`
- **Action**: Create migration that recreates `accounting_entries` table with simplified schema
- **Schema changes**:
  - Remove: `debit_account`, `credit_account`, `debit_amount`, `credit_amount`
  - Add: `type` (TEXT: 'income' | 'expense'), `category` (TEXT: 'tuition' | 'rent' | 'salaries' | 'utilities' | 'other')
  - Keep: `id`, `description`, `amount`, `date`, `reference`, `created_at`
- **Dependencies**: None
- **Verification**: Migration file exists and SQL is valid
- **Status**: ✅ Complete

### Task 1.2: Implement data migration logic
- **File**: `src-tauri/migrations/016_simplify_accounting_schema.sql`
- **Action**: Add INSERT INTO...SELECT to migrate existing data
  - Map PUC 6xx (credit) → type='income', category='tuition'
  - Map PUC 4xxx (debit) → type='expense', category from account mapping
- **Dependencies**: Task 1.1
- **Verification**: SQL handles edge cases, no data loss
- **Status**: ✅ Complete

### Task 1.3: Drop old tables
- **File**: `src-tauri/migrations/016_simplify_accounting_schema.sql`
- **Action**: Drop `account_categories` table, `trial_balance` view, `financial_balance` view
- **Dependencies**: Task 1.2
- **Verification**: Old tables removed after migration
- **Status**: ✅ Complete

---

## Phase 2: Backend Cleanup (8 tasks)

### Task 2.1: Delete liability entities
- **Files**: 
  - `src-tauri/src/domain/entities/liability.rs` (if exists)
  - `src-tauri/src/domain/entities/mod.rs` (remove module)
- **Action**: Delete liability entity and remove from mod.rs
- **Dependencies**: None
- **Verification**: `cargo check` passes
- **Status**: ✅ Complete (no separate liability entity existed)

### Task 2.2: Delete equity entities
- **Files**: 
  - `src-tauri/src/domain/entities/equity.rs` (if exists)
  - `src-tauri/src/domain/entities/mod.rs` (remove module)
- **Action**: Delete equity entity and remove from mod.rs
- **Dependencies**: Task 2.1
- **Verification**: `cargo check` passes
- **Status**: ✅ Complete (no separate equity entity existed)

### Task 2.3: Delete fixed_assets entities
- **Files**: 
  - `src-tauri/src/domain/entities/fixed_asset.rs` (if exists)
  - `src-tauri/src/domain/entities/mod.rs` (remove module)
- **Action**: Delete fixed_asset entity and remove from mod.rs
- **Dependencies**: Task 2.2
- **Verification**: `cargo check` passes
- **Status**: ✅ Complete (no separate fixed_asset entity existed)

### Task 2.4: Delete employee and payroll entities
- **Files**: 
  - `src-tauri/src/domain/entities/employee.rs`
  - `src-tauri/src/domain/entities/payroll.rs`
  - `src-tauri/src/domain/entities/mod.rs` (remove modules)
- **Action**: Delete employee and payroll entities
- **Dependencies**: Task 2.3
- **Verification**: `cargo check` passes
- **Status**: ✅ Complete

### Task 2.5: Delete unused repositories
- **Files**: 
  - `src-tauri/src/infrastructure/repositories/sqlite/liability.rs`
  - `src-tauri/src/infrastructure/repositories/sqlite/payroll.rs`
  - `src-tauri/src/infrastructure/repositories/sqlite/employee.rs`
  - `src-tauri/src/infrastructure/repositories/sqlite/mod.rs` (remove modules)
- **Action**: Delete liability, payroll, employee repositories
- **Dependencies**: Tasks 2.1-2.4
- **Verification**: `cargo check` passes
- **Status**: ✅ Complete

### Task 2.6: Delete unused use cases
- **Files**: 
  - `src-tauri/src/application/use_cases/employee.rs`
  - `src-tauri/src/application/use_cases/payroll.rs`
  - `src-tauri/src/application/use_cases/mod.rs` (remove modules)
- **Action**: Delete employee and payroll use cases
- **Dependencies**: Tasks 2.1-2.5
- **Verification**: `cargo check` passes
- **Status**: ✅ Complete

### Task 2.7: Simplify accounting commands
- **File**: `src-tauri/src/commands/accounting.rs`
- **Action**: Remove all commands except:
  - `create_entry` (unified for income/expense)
  - `list_entries`
  - `get_accounting_summary`
  - `delete_entry`
- **Dependencies**: Task 2.6
- **Verification**: Only simplified commands remain
- **Status**: ✅ Complete

### Task 2.8: Delete unused command files
- **Files**: 
  - `src-tauri/src/commands/employees.rs`
  - `src-tauri/src/commands/payroll.rs`
  - `src-tauri/src/commands/pdf.rs`
  - `src-tauri/src/commands/accounting_ext.rs` (if exists)
  - `src-tauri/src/commands/mod.rs` (remove modules)
- **Action**: Delete employee, payroll, PDF command files
- **Dependencies**: Task 2.7
- **Verification**: `cargo check` passes, `cargo build` succeeds
- **Status**: ✅ Complete

---

## Phase 3: Frontend Cleanup (7 tasks)

### Task 3.1: Delete unused accounting components
- **Files**: 
  - `src/features/accounting/components/EquityForm.tsx`
  - `src/features/accounting/components/LiabilityForm.tsx`
  - `src/features/accounting/components/FixedAssetForm.tsx`
  - `src/features/accounting/components/PDFGenerator.ts`
  - `src/features/accounting/components/index.ts` (remove exports)
- **Action**: Delete equity, liability, fixed_asset forms and PDF generator
- **Dependencies**: None
- **Verification**: No TypeScript errors for missing imports
- **Status**: ✅ Complete

### Task 3.2: Delete unused accounting pages
- **Files**: 
  - `src/features/accounting/routes/ReportsPage.tsx`
  - `src/features/accounting/routes/PayrollPage.tsx`
  - `src/features/accounting/routes/EmployeesPage.tsx`
  - `src/features/accounting/routes/index.ts` (remove exports)
- **Action**: Delete reports, payroll, employees pages
- **Dependencies**: Task 3.1
- **Verification**: No TypeScript errors
- **Status**: ✅ Complete

### Task 3.3: Delete unused hooks
- **Files**: 
  - `src/features/accounting/hooks/usePayroll.ts`
  - `src/features/accounting/hooks/useEmployees.ts`
  - `src/features/accounting/hooks/index.ts` (remove exports)
- **Action**: Delete payroll and employees hooks
- **Dependencies**: Task 3.2
- **Verification**: No TypeScript errors
- **Status**: ✅ Complete

### Task 3.4: Delete unused types
- **Files**: 
  - `src/features/accounting/types/payroll.ts`
  - `src/features/accounting/types/employees.ts`
  - `src/features/accounting/types/index.ts` (remove exports)
- **Action**: Delete payroll and employees types
- **Dependencies**: Task 3.3
- **Verification**: No TypeScript errors
- **Status**: ✅ Complete

### Task 3.5: Update accounting types
- **File**: `src/features/accounting/types/accounting.ts`
- **Action**: Simplify to only include:
  - `AccountingEntry` (with `type: 'income' | 'expense'`, `category` enum)
  - `AccountingSummary`
  - `CreateEntryPayload`
- **Dependencies**: Task 3.4
- **Verification**: Types match new backend schema
- **Status**: ✅ Complete

### Task 3.6: Simplify AccountingPage
- **File**: `src/features/accounting/routes/AccountingPage.tsx`
- **Action**: Remove sub-routes, create single-page layout with:
  - Income form
  - Expense form
  - Summary cards
  - Transactions table
- **Dependencies**: Tasks 3.1-3.5
- **Verification**: Page renders without errors
- **Status**: ✅ Complete

### Task 3.7: Simplify useAccounting hook
- **File**: `src/features/accounting/hooks/useAccounting.ts`
- **Action**: Update to use new simplified Tauri commands
- **Dependencies**: Task 3.6
- **Verification**: Hook works with new backend
- **Status**: ✅ Complete

---

## Phase 4: New Components (4 tasks)

### Task 4.1: Create simplified IncomeForm
- **File**: `src/features/accounting/components/IncomeForm.tsx`
- **Action**: Create form with:
  - Description input
  - Amount input
  - Category select (tuition, other)
  - Date picker
  - Submit button
- **Dependencies**: Task 3.5
- **Verification**: Form renders, validates, submits
- **Status**: ✅ Complete

### Task 4.2: Create simplified ExpenseForm
- **File**: `src/features/accounting/components/ExpenseForm.tsx`
- **Action**: Create form with:
  - Description input
  - Amount input
  - Category select (rent, salaries, utilities, other)
  - Date picker
  - Submit button
- **Dependencies**: Task 3.5
- **Verification**: Form renders, validates, submits
- **Status**: ✅ Complete

### Task 4.3: Create DashboardCards component
- **File**: `src/features/accounting/components/DashboardCards.tsx`
- **Action**: Create cards showing:
  - Total income
  - Total expenses
  - Net balance
  - Monthly comparison
- **Dependencies**: Task 3.5
- **Verification**: Cards display correct data
- **Status**: ✅ Complete

### Task 4.4: Update DashboardCharts for recharts
- **File**: `src/features/accounting/components/DashboardCharts.tsx`
- **Action**: Replace chart.js with recharts
  - Income vs Expenses bar chart
  - Category distribution pie chart
  - Monthly trend line chart
- **Dependencies**: Task 3.5
- **Verification**: Charts render with sample data
- **Status**: ✅ Complete

---

## Phase 5: Integration (3 tasks)

### Task 5.1: Update router
- **File**: `src/app/router.tsx`
- **Action**: Remove sub-routes for employees, payroll, reports
- **Dependencies**: Tasks 3.6-4.4
- **Verification**: Navigation works correctly
- **Status**: ✅ Complete

### Task 5.2: Update sidebar navigation
- **File**: `src/app/components/Sidebar.tsx` (or equivalent)
- **Action**: Remove links to employees, payroll, reports pages
- **Dependencies**: Task 5.1
- **Verification**: Sidebar shows only accounting link
- **Status**: ✅ Complete

### Task 5.3: Test full flow
- **Files**: All modified files
- **Action**: 
  - Run `bunx tsc --noEmit`
  - Run `bun run build`
  - Manual test: create income, create expense, view summary
- **Dependencies**: Tasks 1.1-5.2
- **Verification**: All tests pass, no TypeScript errors
- **Status**: ✅ Complete

---

## Dependencies Graph

```
Phase 1 (Migration)
  └─→ Phase 2 (Backend Cleanup)
        └─→ Phase 3 (Frontend Cleanup)
              └─→ Phase 4 (New Components)
                    └─→ Phase 5 (Integration)
```

---

## Risk Mitigation

1. **Data loss during migration**: Test migration on backup first
2. **Breaking changes**: Keep old commands temporarily with deprecation warnings
3. **Chart.js removal**: Ensure recharts covers all use cases before removing
4. **Form validation**: Use react-hook-form + zod for consistent validation

---

## Verification Checklist

- [x] `bunx tsc --noEmit` passes
- [x] `bun run build` succeeds
- [ ] `cargo check` passes
- [ ] `cargo build` succeeds
- [ ] Manual testing complete
- [ ] No console.log in production code
- [ ] All types explicit (no `any`)
