// LiabilityForm - Form to register debts/pasivos

import { useState } from "react";
import { cn } from "../../../lib/utils";
import type { CreateLiabilityRequest, LiabilityType } from "../types";

interface Props {
  onSubmit: (data: CreateLiabilityRequest) => Promise<void>;
  onCancel: () => void;
}

export function LiabilityForm({ onSubmit, onCancel }: Props) {
  const [formData, setFormData] = useState<Partial<CreateLiabilityRequest>>({
    liability_type: "short_term",
    document_type: "factura",
  });
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.provider_name || !formData.amount || !formData.due_date) {
      alert("Completa los campos requeridos");
      return;
    }

    setSubmitting(true);
    try {
      await onSubmit(formData as CreateLiabilityRequest);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <h3 className="text-lg font-semibold text-slate-900">Registrar Pasivo / Deuda</h3>
      
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Proveedor / Acreedor *
          </label>
          <input
            type="text"
            value={formData.provider_name || ""}
            onChange={(e) => setFormData({ ...formData, provider_name: e.target.value })}
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
            placeholder="Ej: Banco BCP, Proveedor XYZ"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Tipo de Documento
          </label>
          <select
            value={formData.document_type || "factura"}
            onChange={(e) => setFormData({ ...formData, document_type: e.target.value })}
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
          >
            <option value="factura">Factura</option>
            <option value="recibo">Recibo</option>
            <option value="letra">Letra de cambio</option>
            <option value="contrato">Contrato</option>
            <option value="otro">Otro</option>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            N° Documento
          </label>
          <input
            type="text"
            value={formData.document_number || ""}
            onChange={(e) => setFormData({ ...formData, document_number: e.target.value })}
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
            placeholder="F001-12345"
          />
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

        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Tipo de Pasivo *
          </label>
          <select
            value={formData.liability_type || "short_term"}
            onChange={(e) => setFormData({ ...formData, liability_type: e.target.value as LiabilityType })}
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
          >
            <option value="short_term">Corto Plazo (&lt; 1 año)</option>
            <option value="long_term">Largo Plazo (&gt; 1 año)</option>
            <option value="provisions">Provisiones</option>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-slate-700">
            Fecha Vencimiento *
          </label>
          <input
            type="date"
            value={formData.due_date || ""}
            onChange={(e) => setFormData({ ...formData, due_date: e.target.value })}
            required
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
          />
        </div>
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-slate-700">
          Descripción / Notas
        </label>
        <textarea
          value={formData.description || ""}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          rows={2}
          className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
          placeholder="Descripción adicional..."
        />
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
            "rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700",
            submitting && "cursor-not-allowed opacity-50"
          )}
        >
          {submitting ? "Guardando..." : "Registrar Pasivo"}
        </button>
      </div>
    </form>
  );
}