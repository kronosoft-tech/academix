# Proposal: Simplify Accounting Module

## Intent

The current accounting module is a full double-entry bookkeeping system with PUC Colombian/Peruvian chart of accounts (~80 accounts), trial balance, financial balance (A=L+E), fixed assets with depreciation, liability tracking, equity management, employee/payroll management, and PDF export. This is 10x more complex than what's needed.

The goal is to simplify the accounting module to **basic administration**: simple cost, expense, and income tracking with a dashboard showing profit/costs/income/spending breakdown.

## Scope

### In Scope
1. **Simplified Income/Expense Forms**: Keep IncomeForm and ExpenseForm but replace PUC account mapping with simple categories (Income: Tuition, Other / Expense: Rent, Salaries, Utilities, Other)
2. **Dashboard with Charts**: Keep and enhance the existing dashboard (Income vs Expenses bar chart, monthly trend, expense breakdown doughnut, profit margin)
3. **Basic Summary Statistics**: Total income, total expenses, net balance
4. **Income Statement**: Revenue - Expenses = Profit (simplified, no double-entry)

### Out of Scope
- Double-entry bookkeeping / journal entries
- PUC chart of accounts (80+ accounts)
- Trial Balance (Balance de Comprobación)
- Financial Balance (Balance Financiero)
- Fixed Assets with depreciation (remove entirely)
- Liabilities (Pasivos) tracking
- Equity (Patrimonio) management
- PDF export (remove or defer)
- Payment → Accounting auto-sync (remove - payments module handles its own tracking)

## Approach

1. **Remove complexity**: Delete backend services, repositories, entities, and commands related to double-entry accounting, fixed assets, liabilities, equities, trial balance, and financial balance
2. **Simplify frontend**: Remove EquityForm, LiabilityForm, FixedAssetForm, ReportsPage, PDFGenerator. Simplify AccountingPage to show only income/expense forms + dashboard charts
3. **Simplify categories**: Replace PUC account structure with simple enum-based categories (3 income categories, 4 expense categories)
4. **Keep useful components**: AccountingTable (simplified), DashboardCharts, SkeletonTable
5. **Clean up navigation**: Remove "Nómina", "Empleados", "Reportes" from sidebar. Keep single "Contabilidad" entry
6. **Database cleanup**: Remove migrations 011-015 (PUC seed, liabilities, equity, fixed assets). Keep accounting_entries table but simplify it

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/features/accounting/routes/` | Modified | Remove ReportsPage, simplify AccountingPage |
| `src/features/accounting/components/` | Modified | Remove LiabilityForm, EquityForm, FixedAssetForm, PDFGenerator |
| `src/features/accounting/hooks/useAccounting.ts` | Modified | Remove liability, equity, fixed asset, trial balance, financial balance logic |
| `src/features/accounting/types/accounting.ts` | Modified | Simplify types, remove complex accounting types |
| `src-tauri/src/domain/entities/accounting.rs` | Removed | Remove accounting domain entities |
| `src-tauri/src/application/use_cases/accounting.rs` | Removed | Remove accounting service |
| `src-tauri/src/infrastructure/repositories/sqlite/accounting.rs` | Removed | Remove accounting repository |
| `src-tauri/migrations/011-015.sql` | Removed | Remove PUC seed, liabilities, equity, fixed assets migrations |
| `src-tauri/src/commands/accounting.rs` | Modified | Simplify to income/expense CRUD |
| `src/app/layouts/MainLayout.tsx` | Modified | Remove Nominas, Empleados, Reportes from navigation |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Breaking payment history (existing accounting entries reference PUC) | Medium | Remove auto-sync but keep manual entry forms. Existing entries become orphaned but harmless |
| Losing navigation to employee/payroll features | Low | These can remain as separate modules outside accounting if needed later |
| Users expect PUC features | Low | Simplification is the stated goal; complexity can be re-added if needed |

## Rollback Plan

1. Revert all file changes (git checkout)
2. Restore migrations 011-015 if needed
3. Re-add PUC account seeding if double-entry is needed again

## Dependencies

- None

## Success Criteria

- [ ] Single "Contabilidad" navigation entry (no sub-pages)
- [ ] IncomeForm with simple categories (Tuition, Other)
- [ ] ExpenseForm with simple categories (Rent, Salaries, Utilities, Other)
- [ ] Dashboard showing: Total Income, Total Expenses, Net Balance
- [ ] Dashboard charts: Income vs Expenses, Monthly Trend, Expense Breakdown
- [ ] No references to PUC, trial balance, financial balance, fixed assets, liabilities, or equity
- [ ] Backend reduced to simple CRUD for income/expense entries
