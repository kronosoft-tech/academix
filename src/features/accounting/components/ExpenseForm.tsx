// ExpenseForm Component - Phase 9
// Form for recording expense entries

import { useState, useCallback } from "react";
import { cn } from "../../../lib/utils";
import type { CreateEntryRequest, AccountCategory } from "../types";

interface ExpenseFormProps {
  accounts: AccountCategory[];
  onSubmit: (entry: CreateEntryRequest) => Promise<void>;
  onCancel?: () => void;
  className?: string;
}

export function ExpenseForm({
  accounts,
  onSubmit,
  onCancel,
  className,
}: ExpenseFormProps) {
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState({
    date: new Date().toISOString().split("T")[0],
    description: "",
    amount: "",
    debit_account: "",
    credit_account: "",
    reference: "",
  });

  // Filter for expense accounts (4xxx) and cost accounts (5xxx)
  const expenseAccounts = accounts.filter(
    (a) => (a.category_type === "expense" || a.category_type === "cost") && a.active
  );

  // Common credit accounts for expenses (cash, bank, accounts payable)
  const cashBankAccounts = accounts.filter(
    (a) =>
      (a.category_type === "asset" && a.code.startsWith("10")) ||
      a.code.startsWith("14") ||
      a.category_type === "liability"
  );

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!formData.description || !formData.amount || !formData.debit_account) {
        return;
      }

      setLoading(true);
      try {
        await onSubmit({
          date: formData.date,
          description: formData.description,
          amount: parseFloat(formData.amount),
          debit_account: formData.debit_account,
          credit_account: formData.credit_account || cashBankAccounts[0]?.id || "",
          entry_type: "manual",
          reference: formData.reference || `EGR-${Date.now()}`,
        });
        // Reset form
        setFormData({
          date: new Date().toISOString().split("T")[0],
          description: "",
          amount: "",
          debit_account: "",
          credit_account: "",
          reference: "",
        });
      } finally {
        setLoading(false);
      }
    },
    [formData, onSubmit, cashBankAccounts]
  );

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  return (
    <form
      onSubmit={handleSubmit}
      className={cn("space-y-4 rounded-lg border border-slate-200 bg-white p-6", className)}
    >
      <div className="mb-4">
        <h3 className="text-lg font-semibold text-slate-900">Registrar Gasto</h3>
        <p className="text-sm text-slate-500">
          Registra un gasto o egreso de la empresa
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
            placeholder="Pago de servicios básicos - Mayo 2026"
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        {/* Amount */}
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Monto (S/) *
          </label>
          <input
            type="number"
            name="amount"
            value={formData.amount}
            onChange={handleChange}
            placeholder="0.00"
            step="0.01"
            min="0"
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm font-mono focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        {/* Expense Account (Debit) */}
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Cuenta de Gasto *
          </label>
          <select
            name="debit_account"
            value={formData.debit_account}
            onChange={handleChange}
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <option value="">Seleccionar cuenta...</option>
            {expenseAccounts.map((acc) => (
              <option key={acc.id} value={acc.id}>
                {acc.display_code} - {acc.name}
              </option>
            ))}
          </select>
        </div>

        {/* Bank/Cash Account (Credit) */}
        <div className="md:col-span-2">
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Cuenta de Pago (Haber)
          </label>
          <select
            name="credit_account"
            value={formData.credit_account}
            onChange={handleChange}
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <option value="">Seleccionar cuenta (opcional)...</option>
            {cashBankAccounts.map((acc) => (
              <option key={acc.id} value={acc.id}>
                {acc.display_code} - {acc.name}
              </option>
            ))}
          </select>
        </div>
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
          {loading ? "Guardando..." : "Registrar Gasto"}
        </button>
      </div>
    </form>
  );
}