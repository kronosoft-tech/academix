// IncomeForm Component - Simplified
// Form for recording income entries

import { useState, useCallback } from "react";
import { cn } from "../../../lib/utils";
import { INCOME_CATEGORY } from "../types";
import type { IncomeCategory } from "../types";

interface IncomeFormProps {
  onSubmit: (data: { date: string; category: string; description: string; amount: number }) => Promise<void>;
  onCancel?: () => void;
  className?: string;
}

const INCOME_CATEGORIES = [
  { value: INCOME_CATEGORY.TUITION, label: "Matrícula" },
  { value: INCOME_CATEGORY.OTHER, label: "Otros" },
] as const;

export function IncomeForm({ onSubmit, onCancel, className }: IncomeFormProps) {
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState({
    date: new Date().toISOString().split("T")[0],
    category: "tuition" as IncomeCategory,
    description: "",
    amount: "",
  });

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!formData.description || !formData.amount || parseFloat(formData.amount) <= 0) {
        return;
      }

      setLoading(true);
      try {
        await onSubmit({
          date: formData.date,
          category: formData.category,
          description: formData.description,
          amount: parseFloat(formData.amount),
        });
        setFormData({
          date: new Date().toISOString().split("T")[0],
          category: "tuition",
          description: "",
          amount: "",
        });
      } finally {
        setLoading(false);
      }
    },
    [formData, onSubmit]
  );

  return (
    <form
      onSubmit={handleSubmit}
      className={cn("space-y-4 rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6", className)}
    >
      <div className="mb-4">
        <h3 className="text-lg font-semibold text-[var(--color-foreground)]">Nuevo Ingreso</h3>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Fecha
          </label>
          <input
            type="date"
            name="date"
            value={formData.date}
            onChange={handleChange}
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Categoría
          </label>
          <select
            name="category"
            value={formData.category}
            onChange={handleChange}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            {INCOME_CATEGORIES.map((cat) => (
              <option key={cat.value} value={cat.value}>
                {cat.label}
              </option>
            ))}
          </select>
        </div>

        <div className="md:col-span-2">
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Descripción
          </label>
          <input
            type="text"
            name="description"
            value={formData.description}
            onChange={handleChange}
            placeholder="Ej: Mensualidad - Juan Pérez"
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Monto (S/)
          </label>
          <input
            type="number"
            name="amount"
            value={formData.amount}
            onChange={handleChange}
            placeholder="0.00"
            step="0.01"
            min="0.01"
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm font-mono focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>
      </div>

      <div className="flex justify-end gap-3 pt-4">
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="rounded-md border border-[var(--color-foreground)]/30 px-4 py-2 text-sm font-medium text-[var(--color-foreground)] hover:bg-[var(--color-foreground)]/5"
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
          {loading ? "Guardando..." : "Guardar"}
        </button>
      </div>
    </form>
  );
}
