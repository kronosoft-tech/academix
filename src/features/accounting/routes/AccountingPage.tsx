// AccountingPage - Simplified single-page layout

import { useEffect, useState } from "react";
import { useAccounting } from "../hooks";
import { DashboardCards } from "../components/DashboardCards";
import { IncomeExpensesChart, MonthlyTrendChart, ExpenseBreakdownChart, IncomeBreakdownChart } from "../components/DashboardCharts";
import { AccountingTable } from "../components/AccountingTable";
import { IncomeForm } from "../components/IncomeForm";
import { ExpenseForm } from "../components/ExpenseForm";
import { SkeletonCard } from "../components/SkeletonTable";

type ModalType = "income" | "expense" | null;

export default function AccountingPage() {
  const { summary, loading, getSummary, createIncomeEntry, createExpenseEntry, deleteEntry, entries, listEntries } = useAccounting();
  const [period, setPeriod] = useState(() => {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth() - 5, 1).toISOString().split("T")[0];
    const end = now.toISOString().split("T")[0];
    return { start, end };
  });
  const [modalType, setModalType] = useState<ModalType>(null);

  useEffect(() => {
    if (period.start && period.end) {
      getSummary(period.start, period.end);
      listEntries({ date_from: period.start, date_to: period.end });
    }
  }, [period.start, period.end, getSummary, listEntries]);

  const handleCreateEntry = async (data: { date: string; category: string; description: string; amount: number }) => {
    try {
      if (modalType === "income") {
        await createIncomeEntry(data);
      } else {
        await createExpenseEntry(data);
      }
      await getSummary(period.start, period.end);
      await listEntries({ date_from: period.start, date_to: period.end });
      setModalType(null);
    } catch (err) {
      alert("Error al crear entrada: " + (err instanceof Error ? err.message : String(err)));
    }
  };

  const handleDeleteEntry = async (id: string) => {
    if (!confirm("¿Eliminar este asiento?")) return;
    try {
      await deleteEntry(id);
      await getSummary(period.start, period.end);
      await listEntries({ date_from: period.start, date_to: period.end });
    } catch (err) {
      alert("Error al eliminar: " + (err instanceof Error ? err.message : String(err)));
    }
  };

  const monthlyData = summary?.monthly_data || [];
  const expenseBreakdown = summary?.expenses_by_category || [];
  const incomeBreakdown = summary?.income_by_category || [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Contabilidad</h1>
          <p className="text-sm text-[var(--color-foreground)]/60">Resumen financiero</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setModalType("income")}
            className="rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700"
          >
            + Ingreso
          </button>
          <button
            onClick={() => setModalType("expense")}
            className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
          >
            + Gasto
          </button>
        </div>
      </div>

      {/* Period Selector */}
      <div className="flex items-center gap-4 rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-4">
        <span className="text-sm font-medium text-[var(--color-foreground)]">Período:</span>
        <input
          type="date"
          value={period.start}
          onChange={(e) => setPeriod((p) => ({ ...p, start: e.target.value }))}
          className="rounded-md border border-[var(--color-foreground)]/30 px-3 py-1 text-sm"
        />
        <span className="text-[var(--color-foreground)]/40">-</span>
        <input
          type="date"
          value={period.end}
          onChange={(e) => setPeriod((p) => ({ ...p, end: e.target.value }))}
          className="rounded-md border border-[var(--color-foreground)]/30 px-3 py-1 text-sm"
        />
      </div>

      {/* Summary Cards */}
      <DashboardCards
        totalIncome={summary?.total_income || 0}
        totalExpenses={summary?.total_expenses || 0}
        netBalance={summary?.net_balance || 0}
        loading={loading}
      />

      {/* Charts */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {loading ? (
          <>
            <SkeletonCard />
            <SkeletonCard />
          </>
        ) : (
          <>
            <IncomeExpensesChart
              income={summary?.total_income || 0}
              expenses={summary?.total_expenses || 0}
            />
            <MonthlyTrendChart data={monthlyData} />
          </>
        )}
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {loading ? (
          <>
            <SkeletonCard />
            <SkeletonCard />
          </>
        ) : (
          <>
            <IncomeBreakdownChart data={incomeBreakdown} />
            <ExpenseBreakdownChart data={expenseBreakdown} />
          </>
        )}
      </div>

      {/* Entries Table */}
      <div className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
        <h2 className="mb-4 text-lg font-semibold text-[var(--color-foreground)]">Asientos</h2>
        <AccountingTable entries={entries} onDelete={handleDeleteEntry} />
      </div>

      {/* Income Modal */}
      {modalType === "income" && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-lg rounded-lg bg-[var(--color-background)] p-6 shadow-xl">
            <IncomeForm onSubmit={handleCreateEntry} onCancel={() => setModalType(null)} />
          </div>
        </div>
      )}

      {/* Expense Modal */}
      {modalType === "expense" && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-lg rounded-lg bg-[var(--color-background)] p-6 shadow-xl">
            <ExpenseForm onSubmit={handleCreateEntry} onCancel={() => setModalType(null)} />
          </div>
        </div>
      )}
    </div>
  );
}
