// ReportsPage - Phase 14
// Financial reports page with trial balance, financial balance, and PDF export

import { useEffect, useState, useRef } from "react";
import { useAccounting } from "../hooks";
import { scaleIn, animateTableRows } from "../lib/animations";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "../../../lib/utils";
import type { IncomeStatement, FinancialBalance } from "../types";

type ReportType = "income_statement" | "financial_balance";

export default function ReportsPage() {
  const { getIncomeStatement, getFinancialBalance, loading } = useAccounting();
  const [reportType, setReportType] = useState<ReportType>("financial_balance");
  const [period, setPeriod] = useState({ start: "", end: "" });
  const [incomeStatement, setIncomeStatement] = useState<IncomeStatement | null>(null);
  const [financialBalance, setFinancialBalance] = useState<FinancialBalance | null>(null);
  const modalRef = useRef<HTMLDivElement>(null);

  // Set default period
  useEffect(() => {
    const now = new Date();
    const start = new Date(now.getFullYear(), 0, 1).toISOString().split("T")[0];
    const end = now.toISOString().split("T")[0];
    setPeriod({ start, end });
  }, []);

  const handleGenerateReport = async () => {
    if (reportType === "income_statement") {
      const result = await getIncomeStatement(period.start, period.end);
      setIncomeStatement(result);
    } else {
      // Get financial balance from backend
      const result = await getFinancialBalance(period.end);
      setFinancialBalance(result);
    }
    if (modalRef.current) {
      scaleIn(modalRef.current);
    }
    animateTableRows("tbody tr");
  };

  const handleExportPDF = async () => {
    console.log("[DEBUG] handleExportPDF:", reportType);
    
    if (reportType === "financial_balance" && financialBalance) {
      try {
        console.log("[DEBUG] Calling export_financial_balance_pdf...");
        const result = await invoke<string>("export_financial_balance_pdf", { 
          asOfDate: financialBalance.as_of_date 
        });
        console.log("[DEBUG] Result:", result);
        alert(result);
      } catch (err) {
        console.error("[DEBUG] export error:", err);
        alert("Error al exportar: " + err);
      }
    } else if (reportType === "income_statement" && incomeStatement) {
      try {
        console.log("[DEBUG] Calling export_income_statement_pdf...");
        const result = await invoke<string>("export_income_statement_pdf", { 
          periodStart: incomeStatement.period_start,
          periodEnd: incomeStatement.period_end
        });
        console.log("[DEBUG] Result:", result);
        alert(result);
      } catch (err) {
        console.error("[DEBUG] export error:", err);
        alert("Error al exportar: " + err);
      }
    } else {
      alert("Generá el reporte primero");
    }
  };

  // Format currency
  const formatCurrency = (amount: number) => 
    `S/ ${amount.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Reportes Contables</h1>
          <p className="text-sm text-[var(--color-foreground)]/60">Estados financieros y balances</p>
        </div>
      </div>

      {/* Report Type Selector */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <button
          onClick={() => setReportType("income_statement")}
          className={cn(
            "rounded-lg border p-6 text-left transition-all",
            reportType === "income_statement"
              ? "border-blue-500 bg-[var(--color-primary)]/10"
              : "border-[var(--color-foreground)]/20 bg-[var(--color-background)] hover:border-[var(--color-foreground)]/30"
          )}
        >
          <h3 className="font-semibold text-[var(--color-foreground)]">Estado de Resultados</h3>
          <p className="mt-1 text-sm text-[var(--color-foreground)]/60">
            Muestra ingresos, gastos y ganancia/pérdida del período
          </p>
        </button>
        <button
          onClick={() => setReportType("financial_balance")}
          className={cn(
            "rounded-lg border p-6 text-left transition-all",
            reportType === "financial_balance"
              ? "border-blue-500 bg-[var(--color-primary)]/10"
              : "border-[var(--color-foreground)]/20 bg-[var(--color-background)] hover:border-[var(--color-foreground)]/30"
          )}
        >
          <h3 className="font-semibold text-[var(--color-foreground)]">Balance Financiero</h3>
          <p className="mt-1 text-sm text-[var(--color-foreground)]/60">
            Muestra activos, pasivos y patrimonio
          </p>
        </button>
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
        <button
          onClick={handleGenerateReport}
          disabled={loading}
          className={cn(
            "ml-auto rounded-md bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-primary)]",
            loading && "cursor-not-allowed opacity-50"
          )}
        >
          {loading ? "Generando..." : "Generar Reporte"}
        </button>
      </div>

      {/* Preview Section */}
      {incomeStatement && reportType === "income_statement" && (
        <div ref={modalRef} className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-[var(--color-foreground)]">
              Estado de Resultados - {incomeStatement.period_start} a {incomeStatement.period_end}
            </h2>
            <button
              onClick={handleExportPDF}
              className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
            >
              Exportar PDF
            </button>
          </div>

          {/* Summary */}
          <div className="mb-6 grid grid-cols-1 gap-4 md:grid-cols-4">
            <div className="rounded-lg bg-green-50 p-4">
              <p className="text-sm text-green-700">Ingresos Totales</p>
              <p className="text-xl font-bold text-green-700">
                {formatCurrency(incomeStatement.total_income)}
              </p>
            </div>
            <div className="rounded-lg bg-red-50 p-4">
              <p className="text-sm text-red-700">Gastos Totales</p>
              <p className="text-xl font-bold text-red-700">
                {formatCurrency(incomeStatement.total_expenses)}
              </p>
            </div>
            <div className="rounded-lg bg-orange-50 p-4">
              <p className="text-sm text-orange-700">Costos Totales</p>
              <p className="text-xl font-bold text-orange-700">
                {formatCurrency(incomeStatement.total_costs)}
              </p>
            </div>
            <div className={cn("rounded-lg p-4", incomeStatement.is_profitable ? "bg-green-100" : "bg-red-100")}>
              <p className={cn("text-sm", incomeStatement.is_profitable ? "text-green-700" : "text-red-700")}>
                {incomeStatement.is_profitable ? "Utilidad" : "Pérdida"} Neta
              </p>
              <p className={cn("text-xl font-bold", incomeStatement.is_profitable ? "text-green-700" : "text-red-700")}>
                {formatCurrency(incomeStatement.net_result)}
              </p>
            </div>
          </div>

          {/* Income by Category */}
          <div className="mb-6">
            <h3 className="mb-3 font-semibold text-[var(--color-foreground)]">Ingresos por Categoría (6xxx)</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-[var(--color-foreground)]/5">
                  <tr>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Categoría
                    </th>
                    <th className="h-8 px-4 text-right text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Monto
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {incomeStatement.income_by_category.map((cat, idx) => (
                    <tr key={idx} className="hover:bg-[var(--color-foreground)]/5">
                      <td className="px-4 py-2 text-sm text-[var(--color-foreground)]">{cat.category_name}</td>
                      <td className="px-4 py-2 text-right text-sm text-green-600">
                        {formatCurrency(cat.total)}
                      </td>
                    </tr>
                  ))}
                  {incomeStatement.income_by_category.length === 0 && (
                    <tr>
                      <td colSpan={2} className="px-4 py-4 text-center text-sm text-[var(--color-foreground)]/60">
                        No hay ingresos registrados
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Expenses by Category */}
          <div className="mb-6">
            <h3 className="mb-3 font-semibold text-[var(--color-foreground)]">Gastos por Categoría (4xxx)</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-[var(--color-foreground)]/5">
                  <tr>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Categoría
                    </th>
                    <th className="h-8 px-4 text-right text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Monto
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {incomeStatement.expenses_by_category.map((cat, idx) => (
                    <tr key={idx} className="hover:bg-[var(--color-foreground)]/5">
                      <td className="px-4 py-2 text-sm text-[var(--color-foreground)]">{cat.category_name}</td>
                      <td className="px-4 py-2 text-right text-sm text-red-600">
                        {formatCurrency(cat.total)}
                      </td>
                    </tr>
                  ))}
                  {incomeStatement.expenses_by_category.length === 0 && (
                    <tr>
                      <td colSpan={2} className="px-4 py-4 text-center text-sm text-[var(--color-foreground)]/60">
                        No hay gastos registrados
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {financialBalance && reportType === "financial_balance" && (
        <div ref={modalRef} className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-[var(--color-foreground)]">
              Balance Financiero - {financialBalance.as_of_date}
            </h2>
            <button
              onClick={handleExportPDF}
              className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
            >
              Exportar PDF
            </button>
          </div>

          {/* Summary */}
          <div className="mb-6 grid grid-cols-1 gap-4 md:grid-cols-3">
            <div className="rounded-lg bg-[var(--color-primary)]/10 p-4">
              <p className="text-sm text-[var(--color-primary)]">Total Activos</p>
              <p className="text-xl font-bold text-[var(--color-primary)]">
                {formatCurrency(financialBalance.total_assets)}
              </p>
            </div>
            <div className="rounded-lg bg-red-50 p-4">
              <p className="text-sm text-red-700">Total Pasivos</p>
              <p className="text-xl font-bold text-red-700">
                {formatCurrency(financialBalance.total_liabilities)}
              </p>
            </div>
            <div className="rounded-lg bg-green-50 p-4">
              <p className="text-sm text-green-700">Total Patrimonio</p>
              <p className="text-xl font-bold text-green-700">
                {formatCurrency(financialBalance.total_equity)}
              </p>
            </div>
          </div>

          {/* Assets */}
          <div className="mb-6">
            <h3 className="mb-3 font-semibold text-[var(--color-foreground)]">Activos (1xxx)</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-[var(--color-foreground)]/5">
                  <tr>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Código
                    </th>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Cuenta
                    </th>
                    <th className="h-8 px-4 text-right text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Saldo
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {financialBalance.assets.map((cat, idx) => (
                    <tr key={idx} className="hover:bg-[var(--color-foreground)]/5">
                      <td className="px-4 py-2 text-sm font-mono text-[var(--color-foreground)]/80">{cat.account_code}</td>
                      <td className="px-4 py-2 text-sm text-[var(--color-foreground)]">{cat.account_name}</td>
                      <td className="px-4 py-2 text-right text-sm text-[var(--color-primary)]">
                        {formatCurrency(cat.balance)}
                      </td>
                    </tr>
                  ))}
                  {financialBalance.assets.length === 0 && (
                    <tr>
                      <td colSpan={3} className="px-4 py-4 text-center text-sm text-[var(--color-foreground)]/60">
                        No hay activos registrados
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Liabilities */}
          <div className="mb-6">
            <h3 className="mb-3 font-semibold text-[var(--color-foreground)]">Pasivos (2xxx)</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-[var(--color-foreground)]/5">
                  <tr>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Código
                    </th>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Cuenta
                    </th>
                    <th className="h-8 px-4 text-right text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Saldo
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {financialBalance.liabilities.map((cat, idx) => (
                    <tr key={idx} className="hover:bg-[var(--color-foreground)]/5">
                      <td className="px-4 py-2 text-sm font-mono text-[var(--color-foreground)]/80">{cat.account_code}</td>
                      <td className="px-4 py-2 text-sm text-[var(--color-foreground)]">{cat.account_name}</td>
                      <td className="px-4 py-2 text-right text-sm text-red-600">
                        {formatCurrency(cat.balance)}
                      </td>
                    </tr>
                  ))}
                  {financialBalance.liabilities.length === 0 && (
                    <tr>
                      <td colSpan={3} className="px-4 py-4 text-center text-sm text-[var(--color-foreground)]/60">
                        No hay pasivos registrados
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Equity */}
          <div>
            <h3 className="mb-3 font-semibold text-[var(--color-foreground)]">Patrimonio (3xxx) +Resultado</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-[var(--color-foreground)]/5">
                  <tr>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Código
                    </th>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Cuenta
                    </th>
                    <th className="h-8 px-4 text-right text-xs font-medium uppercase text-[var(--color-foreground)]/60">
                      Saldo
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {financialBalance.equity.map((cat, idx) => (
                    <tr key={idx} className="hover:bg-[var(--color-foreground)]/5">
                      <td className="px-4 py-2 text-sm font-mono text-[var(--color-foreground)]/80">{cat.account_code}</td>
                      <td className="px-4 py-2 text-sm text-[var(--color-foreground)]">{cat.account_name}</td>
                      <td className="px-4 py-2 text-right text-sm text-green-600">
                        {formatCurrency(cat.balance)}
                      </td>
                    </tr>
                  ))}
                  {financialBalance.equity.length === 0 && (
                    <tr>
                      <td colSpan={3} className="px-4 py-4 text-center text-sm text-[var(--color-foreground)]/60">
                        No hay patrimonio registrado
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>

          {/* Balance Verification */}
          <div className="mt-6 rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-foreground)]/5 p-4">
            <p className="text-sm text-[var(--color-foreground)]/80">
              <span className="font-semibold">Verificación:</span> Activos = Pasivos + Patrimonio
            </p>
            <p className="text-sm text-[var(--color-foreground)]/80">
              {formatCurrency(financialBalance.total_assets)} = {formatCurrency(financialBalance.total_liabilities)} + {formatCurrency(financialBalance.total_equity)}
            </p>
            <p className={cn("text-sm font-semibold", financialBalance.is_balanced ? "text-green-600" : "text-red-600")}>
              {financialBalance.is_balanced ? "✓ Balance correcto" : "⚠ Balance no cuadra"}
            </p>
          </div>
        </div>
      )}

      {/* Empty State */}
      {!incomeStatement && !financialBalance && (
        <div className="rounded-lg border border-dashed border-[var(--color-foreground)]/30 p-12 text-center">
          <svg
            className="mx-auto h-12 w-12 text-[var(--color-foreground)]/40"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          <h3 className="mt-4 text-sm font-medium text-[var(--color-foreground)]">Sin reporte generado</h3>
          <p className="mt-2 text-sm text-[var(--color-foreground)]/60">
            Selecciona un tipo de reporte y un período para generar
          </p>
        </div>
      )}
    </div>
  );
}