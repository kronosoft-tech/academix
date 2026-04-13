// PayrollPage - Phase 13
// Payroll management page

import { useEffect, useState, useRef } from "react";
import { usePayroll } from "../hooks";
import { SkeletonTable } from "../components/SkeletonTable";
import { PayrollRunsTable, PayrollEntriesTable } from "../components/AccountingTable";
import { PayrollSummaryChart } from "../components/DashboardCharts";
import { scaleIn, animateTableRows } from "../lib/animations";
import { generatePayrollPDF } from "../components/PDFGenerator";
import { cn } from "../../../lib/utils";
import type { PayrollRun } from "../types";

export default function PayrollPage() {
  const { runs, currentRun, listRuns, getRunWithEntries, runPayroll, loading, error } = usePayroll();
  const [showRunForm, setShowRunForm] = useState(false);
  const [runFormData, setRunFormData] = useState({
    period_start: "",
    period_end: "",
    employee_ids: [] as string[],
  });
  const modalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    listRuns();
  }, [listRuns]);

  // Animate on load
  useEffect(() => {
    if (!loading && runs.length > 0) {
      animateTableRows("tbody tr");
    }
  }, [loading, runs]);

  // Animate modal
  useEffect(() => {
    if (showRunForm && modalRef.current) {
      scaleIn(modalRef.current);
    }
  }, [showRunForm]);

  // Set default period
  useEffect(() => {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), 1).toISOString().split("T")[0];
    const end = now.toISOString().split("T")[0];
    setRunFormData((prev) => ({ ...prev, period_start: start, period_end: end }));
  }, []);

  const handleRunPayroll = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await runPayroll({
        period_start: runFormData.period_start,
        period_end: runFormData.period_end,
        employee_ids: runFormData.employee_ids,
        created_by: "admin", // Would come from auth context
      });
      setShowRunForm(false);
      listRuns();
    } catch (err) {
      console.error("Failed to run payroll:", err);
    }
  };

  const handleViewRun = async (run: PayrollRun) => {
    await getRunWithEntries(run.id);
    if (modalRef.current) {
      scaleIn(modalRef.current);
    }
  };

  const handleExportRun = () => {
    if (currentRun) {
      generatePayrollPDF({
        run: currentRun.run,
        entries: currentRun.entries,
      });
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">Nómina</h1>
          <p className="text-sm text-slate-500">Gestión de planillas y payroll</p>
        </div>
        <button
          onClick={() => setShowRunForm(!showRunForm)}
          className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
        >
          {showRunForm ? "Cancelar" : "Ejecutar Nómina"}
        </button>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-red-700">
          {error}
        </div>
      )}

      {/* Run Payroll Form */}
      {showRunForm && (
        <form
          onSubmit={handleRunPayroll}
          className="rounded-lg border border-slate-200 bg-white p-6"
        >
          <h3 className="mb-4 text-lg font-semibold text-slate-900">Ejecutar Nómina</h3>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Fecha Inicio
              </label>
              <input
                type="date"
                value={runFormData.period_start}
                onChange={(e) =>
                  setRunFormData({ ...runFormData, period_start: e.target.value })
                }
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Fecha Fin
              </label>
              <input
                type="date"
                value={runFormData.period_end}
                onChange={(e) =>
                  setRunFormData({ ...runFormData, period_end: e.target.value })
                }
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
          </div>
          <div className="mt-4 flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setShowRunForm(false)}
              className="rounded-md border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
            >
              Cancelar
            </button>
            <button
              type="submit"
              disabled={loading}
              className={cn(
                "rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700",
                loading && "cursor-not-allowed opacity-50"
              )}
            >
              {loading ? "Procesando..." : "Ejecutar Nómina"}
            </button>
          </div>
        </form>
      )}

      {/* Payroll Runs Table */}
      {loading ? (
        <SkeletonTable rows={5} columns={7} />
      ) : (
        <PayrollRunsTable runs={runs} onRowClick={handleViewRun} />
      )}

      {/* View Run Modal */}
      {currentRun && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div
            ref={modalRef}
            className="max-h-[90vh] w-full max-w-6xl overflow-auto rounded-lg bg-white p-6"
          >
            <div className="mb-4 flex items-center justify-between">
              <div>
                <h2 className="text-xl font-bold text-slate-900">
                  Detalle de Nómina - {currentRun.run.period_display}
                </h2>
                <p className="text-sm text-slate-500">
                  Estado: {currentRun.run.status} | Creado:{" "}
                  {new Date(currentRun.run.created_at).toLocaleDateString("es-PE")}
                </p>
              </div>
              <button
                onClick={() => {
                  handleExportRun();
                }}
                className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
              >
                Exportar PDF
              </button>
            </div>

            {/* Summary Cards */}
            <div className="mb-6 grid grid-cols-1 gap-4 md:grid-cols-3">
              <div className="rounded-lg border border-slate-200 bg-white p-4">
                <p className="text-sm text-slate-500">Total Bruto</p>
                <p className="text-xl font-bold text-slate-900">
                  S/ {currentRun.run.total_gross.toFixed(2)}
                </p>
              </div>
              <div className="rounded-lg border border-slate-200 bg-white p-4">
                <p className="text-sm text-slate-500">Total Deducciones</p>
                <p className="text-xl font-bold text-red-600">
                  - S/ {currentRun.run.total_deductions.toFixed(2)}
                </p>
              </div>
              <div className="rounded-lg border border-slate-200 bg-white p-4">
                <p className="text-sm text-slate-500">Total Neto</p>
                <p className="text-xl font-bold text-green-600">
                  S/ {currentRun.run.total_net.toFixed(2)}
                </p>
              </div>
            </div>

            {/* Chart */}
            <PayrollSummaryChart
              gross={currentRun.run.total_gross}
              deductions={currentRun.run.total_deductions}
              net={currentRun.run.total_net}
              className="mb-6"
            />

            {/* Entries Table */}
            <h3 className="mb-4 text-lg font-semibold text-slate-900">Detalle por Empleado</h3>
            <PayrollEntriesTable entries={currentRun.entries} />

            <div className="mt-4 flex justify-end">
              <button
                onClick={() => getRunWithEntries("")}
                className="rounded-md border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
              >
                Cerrar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}