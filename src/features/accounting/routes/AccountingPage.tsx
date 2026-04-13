// AccountingPage - Phase 13
// Main accounting dashboard page

import { useEffect, useRef, useState } from "react";
import { useAccounting } from "../hooks";
import { SkeletonCard } from "../components/SkeletonTable";
import { IncomeExpensesChart, MonthlyTrendChart, ExpenseBreakdownChart, ProfitMarginChart } from "../components/DashboardCharts";
import { fadeInCards, countUp, animateTableRows } from "../lib/animations";
import { generateIncomeStatementPDF } from "../components/PDFGenerator";

export default function AccountingPage() {
  const { summary, getSummary, loading } = useAccounting();
  const [period, setPeriod] = useState({ start: "", end: "" });
  const statsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getSummary();
  }, [getSummary]);

  // Set default period on mount
  useEffect(() => {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split("T")[0];
    const end = now.toISOString().split("T")[0];
    setPeriod({ start, end });
  }, []);

  // Animate on load
  useEffect(() => {
    if (!loading && summary) {
      fadeInCards(".stat-card");
      animateTableRows("#recent-entries tbody tr");
      
      // Animate count up for totals
      const totalDebitsEl = document.getElementById("total-debits");
      const totalCreditsEl = document.getElementById("total-credits");
      if (totalDebitsEl) countUp(totalDebitsEl, 0, summary.total_debits);
      if (totalCreditsEl) countUp(totalCreditsEl, 0, summary.total_credits);
    }
  }, [loading, summary]);

  const handleExportReport = () => {
    if (summary) {
      generateIncomeStatementPDF({
        period_start: period.start,
        period_end: period.end,
        total_income: summary.total_debits, // Simplified
        total_expenses: summary.total_credits, // Simplified
        total_costs: 0,
        net_result: summary.total_debits - summary.total_credits,
        is_profitable: summary.total_debits > summary.total_credits,
        income_by_category: [],
        expenses_by_category: [],
      });
    }
  };

  // Sample data for charts (would come from backend)
  const monthlyData = [
    { month: "Ene", income: 15000, expenses: 10000 },
    { month: "Feb", income: 18000, expenses: 11000 },
    { month: "Mar", income: 22000, expenses: 13000 },
    { month: "Abr", income: 19000, expenses: 12000 },
    { month: "May", income: 25000, expenses: 14000 },
    { month: "Jun", income: 21000, expenses: 11500 },
  ];

  const expenseBreakdown = [
    { category: "Sueldos", amount: 8500 },
    { category: "Servicios", amount: 2100 },
    { category: "Materiales", amount: 1800 },
    { category: "Mantenimiento", amount: 900 },
    { category: "Otros", amount: 700 },
  ];

  const margin = summary ? ((summary.total_debits - summary.total_credits) / summary.total_debits * 100) || 0 : 0;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">Contabilidad</h1>
          <p className="text-sm text-slate-500">Resumen financiero y libros contables</p>
        </div>
        <button
          onClick={handleExportReport}
          className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
        >
          Exportar Reporte
        </button>
      </div>

      {/* Period Selector */}
      <div className="flex items-center gap-4 rounded-lg border border-slate-200 bg-white p-4">
        <span className="text-sm font-medium text-slate-700">Período:</span>
        <input
          type="date"
          value={period.start}
          onChange={(e) => setPeriod((p: { start: string; end: string }) => ({ ...p, start: e.target.value }))}
          className="rounded-md border border-slate-300 px-3 py-1 text-sm"
        />
        <span className="text-slate-400">-</span>
        <input
          type="date"
          value={period.end}
          onChange={(e) => setPeriod((p: { start: string; end: string }) => ({ ...p, end: e.target.value }))}
          className="rounded-md border border-slate-300 px-3 py-1 text-sm"
        />
      </div>

      {/* Stats Cards */}
      <div ref={statsRef} className="grid grid-cols-1 gap-4 md:grid-cols-4">
        {loading ? (
          <>
            <SkeletonCard />
            <SkeletonCard />
            <SkeletonCard />
            <SkeletonCard />
          </>
        ) : (
          <>
            <div className="stat-card rounded-lg border border-slate-200 bg-white p-6">
              <p className="text-sm text-slate-500">Total Débitos</p>
              <p id="total-debits" className="mt-1 text-2xl font-bold text-slate-900">
                S/ 0.00
              </p>
            </div>
            <div className="stat-card rounded-lg border border-slate-200 bg-white p-6">
              <p className="text-sm text-slate-500">Total Créditos</p>
              <p id="total-credits" className="mt-1 text-2xl font-bold text-slate-900">
                S/ 0.00
              </p>
            </div>
            <div className="stat-card rounded-lg border border-slate-200 bg-white p-6">
              <p className="text-sm text-slate-500">Cuentas Activas</p>
              <p className="mt-1 text-2xl font-bold text-slate-900">
                {summary?.account_count ?? 0}
              </p>
            </div>
            <div className="stat-card rounded-lg border border-slate-200 bg-white p-6">
              <p className="text-sm text-slate-500">Asientos Recientes</p>
              <p className="mt-1 text-2xl font-bold text-slate-900">
                {summary?.entry_count ?? 0}
              </p>
            </div>
          </>
        )}
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <IncomeExpensesChart
          income={monthlyData.reduce((sum, d) => sum + d.income, 0)}
          expenses={monthlyData.reduce((sum, d) => sum + d.expenses, 0)}
        />
        <MonthlyTrendChart data={monthlyData} />
      </div>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <ExpenseBreakdownChart data={expenseBreakdown} />
        <ProfitMarginChart margin={margin} />
      </div>

      {/* Recent Entries Table */}
      <div className="rounded-lg border border-slate-200 bg-white p-6">
        <h2 className="mb-4 text-lg font-semibold text-slate-900">Asientos Recientes</h2>
        <div className="overflow-x-auto">
          <table id="recent-entries" className="w-full">
            <thead className="bg-slate-50">
              <tr>
                <th className="h-10 px-4 text-left text-xs font-medium uppercase text-slate-500">
                  Fecha
                </th>
                <th className="h-10 px-4 text-left text-xs font-medium uppercase text-slate-500">
                  Referencia
                </th>
                <th className="h-10 px-4 text-left text-xs font-medium uppercase text-slate-500">
                  Descripción
                </th>
                <th className="h-10 px-4 text-right text-xs font-medium uppercase text-slate-500">
                  Monto
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {summary?.recent_entries?.slice(0, 5).map((entry) => (
                <tr key={entry.id} className="hover:bg-slate-50">
                  <td className="whitespace-nowrap px-4 py-3 text-sm text-slate-900">
                    {new Date(entry.date).toLocaleDateString("es-PE")}
                  </td>
                  <td className="whitespace-nowrap px-4 py-3 text-sm font-mono text-slate-600">
                    {entry.reference}
                  </td>
                  <td className="px-4 py-3 text-sm text-slate-900">{entry.description}</td>
                  <td className="whitespace-nowrap px-4 py-3 text-right text-sm font-medium text-slate-900">
                    S/ {entry.amount.toFixed(2)}
                  </td>
                </tr>
              ))}
              {(!summary?.recent_entries || summary.recent_entries.length === 0) && (
                <tr>
                  <td colSpan={4} className="px-4 py-8 text-center text-slate-500">
                    No hay asientos recientes
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}