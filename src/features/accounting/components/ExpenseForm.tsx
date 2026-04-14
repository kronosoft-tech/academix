// ExpenseForm Component
// Simple form for recording expense entries with category selector

import { useState, useCallback } from "react";
import { cn } from "../../../lib/utils";
import type { CreateEntryRequest } from "../types";

interface ExpenseFormProps {
  onSubmit: (entry: CreateEntryRequest) => Promise<void>;
  onCancel?: () => void;
  className?: string;
}

// Expense type mapping to accounting accounts (PUC)
// Credit: 1105 (Caja) - where money goes out
// Debit: varies by expense type
const EXPENSE_TYPES = [
  { value: "sueldos", label: "Salarios y Nómina", debitAccount: "4105" },
  { value: "transporte", label: "Auxilio de Transporte", debitAccount: "4110" },
  { value: "salud", label: "Aporte a Salud (EPS)", debitAccount: "4150" },
  { value: "pension", label: "Aporte a Pensión (AFP)", debitAccount: "4155" },
  { value: "ar", label: "Aporte a Riesgos Laborales (ARL)", debitAccount: "4160" },
  { value: "icbf_sena", label: "Aporte a ICBF y SENA", debitAccount: "4165" },
  { value: "arriendamiento", label: "Arrendamiento", debitAccount: "4210" },
  { value: "servicios", label: "Servicios", debitAccount: "4220" },
  { value: "mantenimiento", label: "Mantenimiento y Reparaciones", debitAccount: "4240" },
  { value: "otros", label: "Otros Gastos", debitAccount: "4290" },
] as const;

type ExpenseType = (typeof EXPENSE_TYPES)[number]["value"];

export function ExpenseForm({
  onSubmit,
  onCancel,
  className,
}: ExpenseFormProps) {
  const [loading, setLoading] = useState(false);
  const [expenseType, setExpenseType] = useState<ExpenseType>("sueldos");
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

  const getDebitAccount = (): string => {
    const selected = EXPENSE_TYPES.find((t) => t.value === expenseType);
    return selected?.debitAccount || "4105";
  };

  const getExpenseLabel = (): string => {
    const selected = EXPENSE_TYPES.find((t) => t.value === expenseType);
    return selected?.label || "Gastos";
  };

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!formData.description || !formData.amount) {
        return;
      }

      setLoading(true);
      try {
        // Get debit account based on expense type
        const debitAccount = getDebitAccount();

        await onSubmit({
          date: formData.date,
          description: formData.description,
          amount: parseFloat(formData.amount),
          debit_account: debitAccount,
          credit_account: "1105", // Caja
          entry_type: "manual",
          reference: formData.reference || `EGR-${Date.now()}`,
        });
        // Reset form
        setFormData({
          date: new Date().toISOString().split("T")[0],
          description: "",
          amount: "",
          reference: "",
        });
        setExpenseType("sueldos");
        setCustomDescription("");
      } finally {
        setLoading(false);
      }
    },
    [formData, expenseType, customDescription, onSubmit]
  );

  // Show custom description input only for "otros"
  const showCustomDescription = expenseType === "otros";

  return (
    <form
      onSubmit={handleSubmit}
      className={cn("space-y-4 rounded-lg border border-slate-200 bg-white p-6", className)}
    >
      <div className="mb-4">
        <h3 className="text-lg font-semibold text-slate-900">Registrar Gasto</h3>
        <p className="text-sm text-slate-500">
          Registra un gasto de la empresa
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

        {/* Expense Type Selector */}
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Tipo de Gasto
          </label>
          <select
            value={expenseType}
            onChange={(e) => setExpenseType(e.target.value as ExpenseType)}
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            {EXPENSE_TYPES.map((type) => (
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
              Especificar Otro Gasto *
            </label>
            <input
              type="text"
              value={customDescription}
              onChange={(e) => setCustomDescription(e.target.value)}
              placeholder="Ej: Propaganda, seguro, etc."
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
            placeholder="EGR-0001"
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
            placeholder="Ej: Pago de nómina enero"
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
      <div className="rounded-md bg-orange-50 p-3 text-sm text-orange-700">
        <p className="font-medium">Cuentas contables automáticas:</p>
        <ul className="mt-1 list-inside list-disc">
          <li>Débitos (gasto): {getDebitAccount()} - {getExpenseLabel()}</li>
          <li>Créditos (salida): 1105 - Caja</li>
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
            "rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700",
            loading && "cursor-not-allowed opacity-50"
          )}
        >
          {loading ? "Guardando..." : "Registrar Egreso"}
        </button>
      </div>
    </form>
  );
}