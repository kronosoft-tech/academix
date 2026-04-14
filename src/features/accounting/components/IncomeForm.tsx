// IncomeForm Component
// Simple form for recording income entries with category selector

import { useState, useCallback } from "react";
import { cn } from "../../../lib/utils";
import type { CreateEntryRequest } from "../types";

interface IncomeFormProps {
  onSubmit: (entry: CreateEntryRequest) => Promise<void>;
  onCancel?: () => void;
  className?: string;
}

// Income type mapping to accounting accounts (PUC)
// Debit: 1105 (Caja) - where money comes in
// Credit: varies by income type
const INCOME_TYPES = [
  { value: "mensualidades", label: "Mensualidades", creditAccount: "6115" },
  { value: "cursos_especiales", label: "Cursos Especiales", creditAccount: "6120" },
  { value: "uniformes", label: "Venta de Insumos/Uniformes", creditAccount: "6130" },
  { value: "arrendamientos", label: "Arrendamientos", creditAccount: "6215" },
  { value: "otros", label: "Otros Ingresos", creditAccount: "6220" },
] as const;

type IncomeType = (typeof INCOME_TYPES)[number]["value"];

export function IncomeForm({
  onSubmit,
  onCancel,
  className,
}: IncomeFormProps) {
  const [loading, setLoading] = useState(false);
  const [incomeType, setIncomeType] = useState<IncomeType>("mensualidades");
  const [customDescription, setCustomDescription] = useState("");
  const [formData, setFormData] = useState({
    date: new Date().toISOString().split("T")[0],
    description: "",
    amount: "",
    reference: "",
  });

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const getCreditAccount = (): string => {
    const selected = INCOME_TYPES.find((t) => t.value === incomeType);
    return selected?.creditAccount || "6115";
  };

  const getIncomeLabel = (): string => {
    const selected = INCOME_TYPES.find((t) => t.value === incomeType);
    return selected?.label || "Mensualidades";
  };

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!formData.description || !formData.amount) {
        return;
      }

      setLoading(true);
      try {
        // Get credit account based on income type
        const creditAccount = getCreditAccount();

        await onSubmit({
          date: formData.date,
          description: formData.description,
          amount: parseFloat(formData.amount),
          debit_account: "1105", // Caja
          credit_account: creditAccount,
          entry_type: "manual",
          reference: formData.reference || `ING-${Date.now()}`,
        });
        // Reset form
        setFormData({
          date: new Date().toISOString().split("T")[0],
          description: "",
          amount: "",
          reference: "",
        });
        setIncomeType("mensualidades");
        setCustomDescription("");
      } finally {
        setLoading(false);
      }
    },
    [formData, incomeType, customDescription, onSubmit]
  );

  // Show custom description input only for "otros"
  const showCustomDescription = incomeType === "otros";

  return (
    <form
      onSubmit={handleSubmit}
      className={cn("space-y-4 rounded-lg border border-slate-200 bg-white p-6", className)}
    >
      <div className="mb-4">
        <h3 className="text-lg font-semibold text-slate-900">Registrar Ingreso</h3>
        <p className="text-sm text-slate-500">
          Registra ingresos por diferentes conceptos
        </p>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {/* Date */}
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Fecha
          </label>
          <input
            type="date"
            name="date"
            value={formData.date}
            onChange={handleChange}
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        {/* Income Type Selector */}
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Tipo de Ingreso
          </label>
          <select
            value={incomeType}
            onChange={(e) => setIncomeType(e.target.value as IncomeType)}
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            {INCOME_TYPES.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
        </div>

        {/* Custom Description - only for "otros" */}
        {showCustomDescription && (
          <div className="md:col-span-2">
            <label className="mb-1 block text-sm font-medium text-slate-700">
              Especificar Otherro Ingreso *
            </label>
            <input
              type="text"
              value={customDescription}
              onChange={(e) => setCustomDescription(e.target.value)}
              placeholder="Ej: Donación, venta de activo, etc."
              required={showCustomDescription}
              className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
        )}

        {/* Reference */}
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Referencia
          </label>
          <input
            type="text"
            name="reference"
            value={formData.reference}
            onChange={handleChange}
            placeholder="ING-0001"
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm font-mono focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        {/* Description */}
        <div className="md:col-span-2">
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Descripción *
          </label>
          <input
            type="text"
            name="description"
            value={formData.description}
            onChange={handleChange}
            placeholder="Ej: Mensualidad Enero - Juan Pérez"
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        {/* Amount */}
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Monto (COP) *
          </label>
          <input
            type="number"
            name="amount"
            value={formData.amount}
            onChange={handleChange}
            placeholder="0"
            step="0.01"
            min="0"
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm font-mono focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Info box - showing automatic accounts */}
      <div className="rounded-md bg-blue-50 p-3 text-sm text-blue-700">
        <p className="font-medium">Cuentas contables automáticas:</p>
        <ul className="mt-1 list-inside list-disc">
          <li>Débitos (entrada): 1105 - Caja</li>
          <li>Créditos (ingreso): {getCreditAccount()} - {getIncomeLabel()}</li>
        </ul>
      </div>

      <div className="flex justify-end gap-3 pt-4">
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="rounded-md border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
          >
            Cancelar
          </button>
        )}
        <button
          type="submit"
          disabled={loading}
          className={cn(
            "rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700",
            loading && "cursor-not-allowed opacity-50"
          )}
        >
          {loading ? "Guardando..." : "Registrar Ingreso"}
        </button>
      </div>
    </form>
  );
}