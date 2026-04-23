// EquityForm - Form to register equity/patrimonio

import { useState } from "react";
import { cn } from "../../../lib/utils";
import type { CreateEquityRequest, EquityType } from "../types";

interface Props {
  onSubmit: (data: CreateEquityRequest) => Promise<void>;
  onCancel: () => void;
}

export function EquityForm({ onSubmit, onCancel }: Props) {
  const [formData, setFormData] = useState<Partial<CreateEquityRequest>>({
    equity_type: "capital",
  });
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.equity_type || !formData.description || !formData.amount) {
      alert("Completa los campos requeridos");
      return;
    }

    setSubmitting(true);
    try {
      await onSubmit(formData as CreateEquityRequest);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <h3 className="text-lg font-semibold text-slate-900">Registrar Patrimonio</h3>
      
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Tipo de Patrimonio *
          </label>
          <select
            value={formData.equity_type || "capital"}
            onChange={(e) => setFormData({ ...formData, equity_type: e.target.value as EquityType })}
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
          >
            <option value="capital">Capital Social (3105)</option>
            <option value="reserves">Reservas (3305)</option>
            <option value="results">Resultados del Ejercicio (3605)</option>
            <option value="retained">Resultados Acumulados (3610)</option>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Monto (S/) *
          </label>
          <input
            type="number"
            step="0.01"
            min="0"
            value={formData.amount || ""}
            onChange={(e) => setFormData({ ...formData, amount: parseFloat(e.target.value) })}
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
          />
        </div>
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700">
          Descripción *
        </label>
        <input
          type="text"
          value={formData.description || ""}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          required
          className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
          placeholder={
            formData.equity_type === "capital"
              ? "Ej: Aporte inicial de socios"
              : formData.equity_type === "reserves"
              ? "Ej: Reserva legal 10%"
              : formData.equity_type === "results"
              ? "Ej: Utilidad del período 2024"
              : "Ej: Utilidades acumuladas ejercicios anteriores"
          }
        />
      </div>

      <div className="rounded-lg bg-green-50 p-3">
        <p className="text-sm text-green-700">
          <strong>Nota:</strong> El patrimonio representa los recursos propios de la empresa.
          Se incluye en el Balance Financiero: <strong>Activos = Pasivos + Patrimonio</strong>
        </p>
      </div>

      <div className="flex justify-end gap-3 pt-4">
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
        >
          Cancelar
        </button>
        <button
          type="submit"
          disabled={submitting}
          className={cn(
            "rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700",
            submitting && "cursor-not-allowed opacity-50"
          )}
        >
          {submitting ? "Guardando..." : "Registrar Patrimonio"}
        </button>
      </div>
    </form>
  );
}