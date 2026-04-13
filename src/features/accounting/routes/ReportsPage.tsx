// ReportsPage - Phase 13
// Financial reports page with trial balance, income statement, and PDF export

import { useEffect, useState, useRef } from "react";
import { useAccounting } from "../hooks";
import { scaleIn, animateTableRows } from "../lib/animations";
import { generateTrialBalancePDF, generateIncomeStatementPDF } from "../components/PDFGenerator";
import { cn } from "../../../lib/utils";

type ReportType = "trial_balance" | "income_statement";

export default function ReportsPage() {
  const { getTrialBalance, getIncomeStatement, loading } = useAccounting();
  const [reportType, setReportType] = useState<ReportType>("trial_balance");
  const [period, setPeriod] = useState({ start: "", end: "" });
  const [trialBalance, setTrialBalance] = useState<any>(null);
  const [incomeStatement, setIncomeStatement] = useState<any>(null);
  const modalRef = useRef<HTMLDivElement>(null);

  // Set default period
  useEffect(() => {
    const now = new Date();
    const start = new Date(now.getFullYear(), 0, 1).toISOString().split("T")[0];
    const end = now.toISOString().split("T")[0];
    setPeriod({ start, end });
  }, []);

  const handleGenerateReport = async () => {
    if (reportType === "trial_balance") {
      const result = await getTrialBalance(period.end);
      setTrialBalance(result);
    } else {
      const result = await getIncomeStatement(period.start, period.end);
      setIncomeStatement(result);
    }
    if (modalRef.current) {
      scaleIn(modalRef.current);
    }
    animateTableRows("tbody tr");
  };

  const handleExportPDF = () => {
    if (reportType === "trial_balance" && trialBalance) {
      generateTrialBalancePDF(trialBalance);
    } else if (reportType === "income_statement" && incomeStatement) {
      generateIncomeStatementPDF(incomeStatement);
    }
  };

  // Sample data when backend is not connected (for demo)
  const sampleTrialBalance = {
    as_of_date: period.end || new Date().toISOString().split("T")[0],
    accounts: [
      { account_code: "10101", account_name: "Caja", debit_balance: 5000, credit_balance: 0, balance_type: "debit" },
      { account_code: "10102", account_name: "Banco", debit_balance: 15000, credit_balance: 0, balance_type: "debit" },
      { account_code: "12121", account_name: "Cuentas por Cobrar", debit_balance: 8000, credit_balance: 0, balance_type: "debit" },
      { account_code: "20101", account_name: "Proveedores", debit_balance: 0, credit_balance: 3500, balance_type: "credit" },
      { account_code: "40111", account_name: "Ventas", debit_balance: 0, credit_balance: 25000, balance_type: "credit" },
      { account_code: "50111", account_name: "Costos de Ventas", debit_balance: 12000, credit_balance: 0, balance_type: "debit" },
      { account_code: "60111", account_name: "Sueldos", debit_balance: 8500, credit_balance: 0, balance_type: "debit" },
      { account_code: "60112", account_name: "Servicios", debit_balance: 2100, credit_balance: 0, balance_type: "debit" },
    ],
    total_debits: 50600,
    total_credits: 28500,
    is_balanced: false,
  };

  const sampleIncomeStatement = {
    period_start: period.start,
    period_end: period.end,
    total_income: 25000,
    total_expenses: 10600,
    total_costs: 12000,
    net_result: 2400,
    is_profitable: true,
    income_by_category: [
      { category_id: "1", category_name: "Servicios Educativos", total: 22000 },
      { category_id: "2", category_name: "Matrículas", total: 3000 },
    ],
    expenses_by_category: [
      { category_id: "1", category_name: "Sueldos y Salarios", total: 8500 },
      { category_id: "2", category_name: "Servicios Básicos", total: 2100 },
    ],
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">Reportes Contables</h1>
          <p className="text-sm text-slate-500">Estados financieros y balances</p>
        </div>
      </div>

      {/* Report Type Selector */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <button
          onClick={() => setReportType("trial_balance")}
          className={cn(
            "rounded-lg border p-6 text-left transition-all",
            reportType === "trial_balance"
              ? "border-blue-500 bg-blue-50"
              : "border-slate-200 bg-white hover:border-slate-300"
          )}
        >
          <h3 className="font-semibold text-slate-900">Balance de Comprobación</h3>
          <p className="mt-1 text-sm text-slate-500">
            Lista todas las cuentas con sus débitos y créditos
          </p>
        </button>
        <button
          onClick={() => setReportType("income_statement")}
          className={cn(
            "rounded-lg border p-6 text-left transition-all",
            reportType === "income_statement"
              ? "border-blue-500 bg-blue-50"
              : "border-slate-200 bg-white hover:border-slate-300"
          )}
        >
          <h3 className="font-semibold text-slate-900">Estado de Resultados</h3>
          <p className="mt-1 text-sm text-slate-500">
            Muestra ingresos, gastos y resultado del ejercicio
          </p>
        </button>
      </div>

      {/* Period Selector */}
      <div className="flex items-center gap-4 rounded-lg border border-slate-200 bg-white p-4">
        <span className="text-sm font-medium text-slate-700">Período:</span>
        <input
          type="date"
          value={period.start}
          onChange={(e) => setPeriod((p) => ({ ...p, start: e.target.value }))}
          className="rounded-md border border-slate-300 px-3 py-1 text-sm"
        />
        <span className="text-slate-400">-</span>
        <input
          type="date"
          value={period.end}
          onChange={(e) => setPeriod((p) => ({ ...p, end: e.target.value }))}
          className="rounded-md border border-slate-300 px-3 py-1 text-sm"
        />
        <button
          onClick={handleGenerateReport}
          disabled={loading}
          className={cn(
            "ml-auto rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700",
            loading && "cursor-not-allowed opacity-50"
          )}
        >
          {loading ? "Generando..." : "Generar Reporte"}
        </button>
      </div>

      {/* Preview Section */}
      {trialBalance && reportType === "trial_balance" && (
        <div ref={modalRef} className="rounded-lg border border-slate-200 bg-white p-6">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-slate-900">
              Balance de Comprobación - {trialBalance.as_of_date}
            </h2>
            <button
              onClick={handleExportPDF}
              className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
            >
              Exportar PDF
            </button>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-slate-50">
                <tr>
                  <th className="h-10 px-4 text-left text-xs font-medium uppercase text-slate-500">
                    Código
                  </th>
                  <th className="h-10 px-4 text-left text-xs font-medium uppercase text-slate-500">
                    Cuenta
                  </th>
                  <th className="h-10 px-4 text-right text-xs font-medium uppercase text-slate-500">
                    Débitos
                  </th>
                  <th className="h-10 px-4 text-right text-xs font-medium uppercase text-slate-500">
                    Créditos
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {sampleTrialBalance.accounts.map((acc: any) => (
                  <tr key={acc.account_code} className="hover:bg-slate-50">
                    <td className="whitespace-nowrap px-4 py-3 text-sm font-mono text-slate-600">
                      {acc.account_code}
                    </td>
                    <td className="px-4 py-3 text-sm text-slate-900">{acc.account_name}</td>
                    <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-slate-600">
                      {acc.debit_balance > 0 ? `S/ ${acc.debit_balance.toFixed(2)}` : "-"}
                    </td>
                    <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-slate-600">
                      {acc.credit_balance > 0 ? `S/ ${acc.credit_balance.toFixed(2)}` : "-"}
                    </td>
                  </tr>
                ))}
                <tr className="font-bold">
                  <td colSpan={2} className="px-4 py-3 text-right text-sm text-slate-900">
                    TOTALES
                  </td>
                  <td className="px-4 py-3 text-right text-sm text-slate-900">
                    S/ {sampleTrialBalance.total_debits.toFixed(2)}
                  </td>
                  <td className="px-4 py-3 text-right text-sm text-slate-900">
                    S/ {sampleTrialBalance.total_credits.toFixed(2)}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      )}

      {incomeStatement && reportType === "income_statement" && (
        <div ref={modalRef} className="rounded-lg border border-slate-200 bg-white p-6">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-slate-900">
              Estado de Resultados - {period.start} a {period.end}
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
              <p className="text-sm text-green-700">Total Ingresos</p>
              <p className="text-xl font-bold text-green-700">
                S/ {sampleIncomeStatement.total_income.toFixed(2)}
              </p>
            </div>
            <div className="rounded-lg bg-red-50 p-4">
              <p className="text-sm text-red-700">Total Gastos</p>
              <p className="text-xl font-bold text-red-700">
                S/ {sampleIncomeStatement.total_expenses.toFixed(2)}
              </p>
            </div>
            <div className="rounded-lg bg-amber-50 p-4">
              <p className="text-sm text-amber-700">Total Costos</p>
              <p className="text-xl font-bold text-amber-700">
                S/ {sampleIncomeStatement.total_costs.toFixed(2)}
              </p>
            </div>
            <div
              className={cn(
                "rounded-lg p-4",
                sampleIncomeStatement.is_profitable ? "bg-green-50" : "bg-red-50"
              )}
            >
              <p className={cn("text-sm", sampleIncomeStatement.is_profitable ? "text-green-700" : "text-red-700")}>
                Resultado
              </p>
              <p
                className={cn(
                  "text-xl font-bold",
                  sampleIncomeStatement.is_profitable ? "text-green-700" : "text-red-700"
                )}
              >
                {sampleIncomeStatement.is_profitable ? "UTILIDAD" : "PÉRDIDA"}: S/{" "}
                {Math.abs(sampleIncomeStatement.net_result).toFixed(2)}
              </p>
            </div>
          </div>

          {/* Income by Category */}
          <div className="mb-6">
            <h3 className="mb-3 font-semibold text-slate-900">Ingresos por Categoría</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-slate-50">
                  <tr>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-slate-500">
                      Categoría
                    </th>
                    <th className="h-8 px-4 text-right text-xs font-medium uppercase text-slate-500">
                      Total
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {sampleIncomeStatement.income_by_category.map((cat: any) => (
                    <tr key={cat.category_id} className="hover:bg-slate-50">
                      <td className="px-4 py-2 text-sm text-slate-900">{cat.category_name}</td>
                      <td className="px-4 py-2 text-right text-sm text-green-600">
                        S/ {cat.total.toFixed(2)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {/* Expenses by Category */}
          <div>
            <h3 className="mb-3 font-semibold text-slate-900">Gastos por Categoría</h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead className="bg-slate-50">
                  <tr>
                    <th className="h-8 px-4 text-left text-xs font-medium uppercase text-slate-500">
                      Categoría
                    </th>
                    <th className="h-8 px-4 text-right text-xs font-medium uppercase text-slate-500">
                      Total
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {sampleIncomeStatement.expenses_by_category.map((cat: any) => (
                    <tr key={cat.category_id} className="hover:bg-slate-50">
                      <td className="px-4 py-2 text-sm text-slate-900">{cat.category_name}</td>
                      <td className="px-4 py-2 text-right text-sm text-red-600">
                        S/ {cat.total.toFixed(2)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Empty State */}
      {!trialBalance && !incomeStatement && (
        <div className="rounded-lg border border-dashed border-slate-300 p-12 text-center">
          <svg
            className="mx-auto h-12 w-12 text-slate-400"
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
          <h3 className="mt-4 text-sm font-medium text-slate-900">Sin reporte generado</h3>
          <p className="mt-2 text-sm text-slate-500">
            Selecciona un tipo de reporte y un período para generar
          </p>
        </div>
      )}
    </div>
  );
}