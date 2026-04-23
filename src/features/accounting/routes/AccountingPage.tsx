// AccountingPage - Phase 13
// Main accounting dashboard page

import { useEffect, useRef, useState } from "react";
import { useAccounting } from "../hooks";
import { usePayments } from "../../payments/hooks/usePayments";
import { SkeletonCard } from "../components/SkeletonTable";
import { IncomeExpensesChart, MonthlyTrendChart, ExpenseBreakdownChart, ProfitMarginChart } from "../components/DashboardCharts";
import { fadeInCards, countUp, animateTableRows } from "../lib/animations";
import { invoke } from "@tauri-apps/api/core";
import { IncomeForm } from "../components/IncomeForm";
import { ExpenseForm } from "../components/ExpenseForm";
import { LiabilityForm } from "../components/LiabilityForm";
import { EquityForm } from "../components/EquityForm";
import { FixedAssetForm } from "../components/FixedAssetForm";
import type { CreateLiabilityRequest, CreateEquityRequest, CreateFixedAssetRequest } from "../types";

type ModalType = "income" | "expense" | "liability" | "equity" | "fixed_asset" | null;

export default function AccountingPage() {
  const { summary, getSummary, loading, createEntry, listLiabilities, createLiability, listEquities, createEquity, createFixedAsset, liabilities, equities } = useAccounting();
  const { syncPaymentsToAccounting } = usePayments();
  const [period, setPeriod] = useState({ start: "", end: "" });
  const [modalType, setModalType] = useState<ModalType>(null);
  const [syncing, setSyncing] = useState(false);
  const statsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getSummary();
    listLiabilities();
    listEquities();
  }, [getSummary, listLiabilities, listEquities]);

  // Set default period on mount
  useEffect(() => {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split("T")[0];
    const end = now.toISOString().split("T")[0];
    setPeriod({ start, end });
  }, []);

  const handleCreateEntry = async (entry: unknown) => {
    try {
      await createEntry(entry as never);
      await getSummary();
      setModalType(null);
    } catch (err) {
      console.error("Error creating entry:", err);
      alert("Error al crear entrada: " + (err instanceof Error ? err.message : String(err)));
    }
  };

  const handleCreateLiability = async (data: CreateLiabilityRequest) => {
    try {
      await createLiability(data);
      await listLiabilities();
      setModalType(null);
      alert("Pasivo registrado exitosamente");
    } catch (err) {
      alert("Error: " + (err instanceof Error ? err.message : String(err)));
    }
  };

  const handleCreateEquity = async (data: CreateEquityRequest) => {
    try {
      await createEquity(data);
      await listEquities();
      setModalType(null);
      alert("Patrimonio registrado exitosamente");
    } catch (err) {
      alert("Error: " + (err instanceof Error ? err.message : String(err)));
    }
  };

  const handleCreateFixedAsset = async (data: CreateFixedAssetRequest) => {
    try {
      await createFixedAsset(data);
      await getSummary(); // Refresh to see new asset in balance
      setModalType(null);
      alert("Activo fijo registrado exitosamente");
    } catch (err) {
      alert("Error: " + (err instanceof Error ? err.message : String(err)));
    }
  };

  // Animate on load
  useEffect(() => {
    if (!loading && summary) {
      fadeInCards(".stat-card");
      animateTableRows("#recent-entries tbody tr");
      
      // Animate count up for totals
      const totalIncomeEl = document.getElementById("total-income");
      const totalExpensesEl = document.getElementById("total-expenses");
      const netBalanceEl = document.getElementById("net-balance");
      if (totalIncomeEl) countUp(totalIncomeEl, 0, summary.total_income);
      if (totalExpensesEl) countUp(totalExpensesEl, 0, summary.total_expenses);
      if (netBalanceEl) countUp(netBalanceEl, 0, summary.net_balance);
    }
  }, [loading, summary]);

const handleExportReport = async () => {
      try {
        const asOfDate = period.end || new Date().toISOString().split("T")[0];
        console.log("[DEBUG] Exporting Balance Financiero PDF for date:", asOfDate);
        const result = await invoke<string>("export_financial_balance_pdf", { asOfDate });
        console.log("[DEBUG] Result:", result);
        alert(result);
      } catch (err) {
        console.error("Error generating report:", err);
        alert("Error al generar reporte: " + (err instanceof Error ? err.message : String(err)));
      }
    };

// Monthly data from summary
   const monthlyData = summary?.monthly_data || [
     { month: "Ene", income: 0, expenses: 0 },
     { month: "Feb", income: 0, expenses: 0 },
     { month: "Mar", income: 0, expenses: 0 },
     { month: "Abr", income: 0, expenses: 0 },
     { month: "May", income: 0, expenses: 0 },
     { month: "Jun", income: 0, expenses: 0 },
   ];

const expenseBreakdown = summary?.expenses_by_category || [
     { category_name: "Sin datos", amount: 0 },
    ];

const margin = summary && summary.total_income > 0 
      ? ((summary.total_income - summary.total_expenses) / summary.total_income * 100) 
      : 0;

  const handleSyncPayments = async () => {
    setSyncing(true);
    try {
      const result = await syncPaymentsToAccounting();
      if (result.success) {
        alert(`Sincronización completa: ${result.synced} pagos sincronizados, ${result.skipped} omitidos`);
        getSummary();
      } else {
        alert(`Error: ${result.error}`);
      }
    } finally {
      setSyncing(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">Contabilidad</h1>
          <p className="text-sm text-slate-500">Resumen financiero y libros contables</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleSyncPayments}
            disabled={syncing}
            className="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-700 disabled:opacity-50"
          >
            {syncing ? "Sincronizando..." : "Sincronizar Pagos"}
          </button>
          <button
            onClick={() => setModalType("expense")}
            className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
          >
            + Gasto
          </button>
          <button
            onClick={() => setModalType("income")}
            className="rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700"
          >
            + Ingreso
          </button>
          <button
            onClick={() => setModalType("liability")}
            className="rounded-lg bg-orange-600 px-4 py-2 text-sm font-medium text-white hover:bg-orange-700"
          >
            + Pasivo
          </button>
          <button
            onClick={() => setModalType("equity")}
            className="rounded-lg bg-purple-600 px-4 py-2 text-sm font-medium text-white hover:bg-purple-700"
          >
            + Patrimonio
          </button>
          <button
            onClick={() => setModalType("fixed_asset")}
            className="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700"
          >
            + Activo Fijo
          </button>
          <button
            onClick={handleExportReport}
            className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
          >
            Exportar
          </button>
        </div>
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
               <p className="text-sm text-slate-500">Total Ingresos</p>
               <p id="total-income" className="mt-1 text-2xl font-bold text-slate-900">
                 S/ {summary?.total_income?.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) ?? '0.00'}
               </p>
               <button
                 className="absolute top-2 right-2 text-xs text-blue-500 hover:text-blue-700"
                 title="Total de dinero que ha entrado por ingresos (cuentas 6xxx)"
               >
                 ?
               </button>
             </div>
             <div className="stat-card rounded-lg border border-slate-200 bg-white p-6">
               <p className="text-sm text-slate-500">Total Gastos</p>
               <p id="total-expenses" className="mt-1 text-2xl font-bold text-slate-900">
                 S/ {summary?.total_expenses?.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) ?? '0.00'}
               </p>
               <button
                 className="absolute top-2 right-2 text-xs text-blue-500 hover:text-blue-700"
                 title="Total de dinero gastado en el periodo (cuentas 4xxx y 5xxx)"
               >
                 ?
               </button>
             </div>
             <div className="stat-card rounded-lg border border-slate-200 bg-white p-6">
               <p className="text-sm text-slate-500">Balance Neto</p>
               <p id="net-balance" className="mt-1 text-2xl font-bold text-slate-900">
                 S/ {summary?.net_balance?.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) ?? '0.00'}
               </p>
               <button
                 className="absolute top-2 right-2 text-xs text-blue-500 hover:text-blue-700"
                 title="Ingresos - Gastos (positivo = ganancia, negativo = pérdida)"
               >
                 ?
               </button>
             </div>
             <div className="stat-card rounded-lg border border-slate-200 bg-white p-6">
               <p className="text-sm text-slate-500">Cuentas Activas</p>
               <p className="mt-1 text-2xl font-bold text-slate-900">
                 {summary?.account_count ?? 0}
               </p>
               <button
                 className="absolute top-2 right-2 text-xs text-blue-500 hover:text-blue-700"
                 title="Cantidad de cuentas contables activas del Plan Único de Cuentas"
               >
                 ?
               </button>
             </div>
           </>
         )}
       </div>

       {/* Charts Grid */}
       <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
         <IncomeExpensesChart
           income={summary?.total_income || 0}
           expenses={summary?.total_expenses || 0}
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

      {/* Income Modal */}
      {modalType === "income" && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
            <IncomeForm
              onSubmit={handleCreateEntry}
              onCancel={() => setModalType(null)}
            />
          </div>
        </div>
      )}

      {/* Expense Modal */}
      {modalType === "expense" && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
            <ExpenseForm
              onSubmit={handleCreateEntry}
              onCancel={() => setModalType(null)}
            />
          </div>
        </div>
      )}

      {/* Liability Modal (Pasivos) */}
      {modalType === "liability" && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
            <button
              onClick={() => setModalType(null)}
              className="absolute right-4 top-4 text-slate-400 hover:text-slate-600"
            >
              ✕
            </button>
            <LiabilityForm
              onSubmit={handleCreateLiability}
              onCancel={() => setModalType(null)}
            />
          </div>
        </div>
      )}

      {/* Equity Modal (Patrimonio) */}
      {modalType === "equity" && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
            <button
              onClick={() => setModalType(null)}
              className="absolute right-4 top-4 text-slate-400 hover:text-slate-600"
            >
              ✕
            </button>
            <EquityForm
              onSubmit={handleCreateEquity}
              onCancel={() => setModalType(null)}
            />
          </div>
        </div>
      )}

      {/* Fixed Asset Modal (Activo Fijo) */}
      {modalType === "fixed_asset" && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl">
            <button
              onClick={() => setModalType(null)}
              className="absolute right-4 top-4 text-slate-400 hover:text-slate-600"
            >
              ✕
            </button>
            <FixedAssetForm
              onSubmit={handleCreateFixedAsset}
              onCancel={() => setModalType(null)}
            />
          </div>
        </div>
      )}

      {/* Pasivos List */}
      {liabilities.length > 0 && (
        <div className="rounded-lg border border-orange-200 bg-orange-50 p-6">
          <h3 className="mb-4 text-lg font-semibold text-orange-900">Pasivos Registrados</h3>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-orange-100">
                <tr>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase text-orange-700">Proveedor</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase text-orange-700">Documento</th>
                  <th className="px-4 py-2 text-right text-xs font-medium uppercase text-orange-700">Monto</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase text-orange-700">Tipo</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase text-orange-700">Vencimiento</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-orange-100">
                {liabilities.map((l) => (
                  <tr key={l.id} className="bg-white">
                    <td className="px-4 py-2 text-sm text-slate-900">{l.provider_name}</td>
                    <td className="px-4 py-2 text-sm text-slate-600">{l.document_number}</td>
                    <td className="px-4 py-2 text-right text-sm font-medium text-red-600">S/ {l.amount.toFixed(2)}</td>
                    <td className="px-4 py-2 text-sm text-slate-600">
                      {l.liability_type === "short_term" ? "Corto Plazo" : l.liability_type === "long_term" ? "Largo Plazo" : "Provisiones"}
                    </td>
                    <td className="px-4 py-2 text-sm text-slate-600">{l.due_date}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Patrimonio List */}
      {equities.length > 0 && (
        <div className="rounded-lg border border-purple-200 bg-purple-50 p-6">
          <h3 className="mb-4 text-lg font-semibold text-purple-900">Patrimonio Registrado</h3>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-purple-100">
                <tr>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase text-purple-700">Tipo</th>
                  <th className="px-4 py-2 text-left text-xs font-medium uppercase text-purple-700">Descripción</th>
                  <th className="px-4 py-2 text-right text-xs font-medium uppercase text-purple-700">Monto</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-purple-100">
                {equities.map((e) => (
                  <tr key={e.id} className="bg-white">
                    <td className="px-4 py-2 text-sm text-slate-900">
                      {e.equity_type === "capital" ? "Capital Social" : e.equity_type === "reserves" ? "Reservas" : e.equity_type === "results" ? "Resultados" : "Resultados Acumulados"}
                    </td>
                    <td className="px-4 py-2 text-sm text-slate-600">{e.description}</td>
                    <td className="px-4 py-2 text-right text-sm font-medium text-green-600">S/ {e.amount.toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}