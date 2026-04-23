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
      <h3 className="text-lg font-semibold text-[var(--color-foreground)]">Registrar Pasivo / Deuda</h3>
      
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Proveedor / Acreedor *
          </label>
          <input
            type="text"
            value={formData.provider_name || ""}
            onChange={(e) => setFormData({ ...formData, provider_name: e.target.value })}
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
            placeholder="Ej: Banco BCP, Proveedor XYZ"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Tipo de Documento
          </label>
          <select
            value={formData.document_type || "factura"}
            onChange={(e) => setFormData({ ...formData, document_type: e.target.value })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          >
            <option value="factura">Factura</option>
            <option value="recibo">Recibo</option>
            <option value="letra">Letra de cambio</option>
            <option value="contrato">Contrato</option>
            <option value="otro">Otro</option>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            N° Documento
          </label>
          <input
            type="text"
            value={formData.document_number || ""}
            onChange={(e) => setFormData({ ...formData, document_number: e.target.value })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
            placeholder="F001-12345"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Monto (S/) *
          </label>
          <input
            type="number"
            step="0.01"
            min="0"
            value={formData.amount || ""}
            onChange={(e) => setFormData({ ...formData, amount: parseFloat(e.target.value) })}
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Tipo de Pasivo *
          </label>
          <select
            value={formData.liability_type || "short_term"}
            onChange={(e) => setFormData({ ...formData, liability_type: e.target.value as LiabilityType })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          >
            <option value="short_term">Corto Plazo (&lt; 1 año)</option>
            <option value="long_term">Largo Plazo (&gt; 1 año)</option>
            <option value="provisions">Provisiones</option>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Es para *
          </label>
          <select
            value={formData.for_type || "expense"}
            onChange={(e) => setFormData({ ...formData, for_type: e.target.value as "expense" | "asset" })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          >
            <option value="expense">Gasto/Servicio (cuenta 4xxx)</option>
            <option value="asset">Activo Fijo (cuenta 16xxx)</option>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Cuenta Contable (Débito) *
          </label>
          <select
            value={formData.debit_account_code || (formData.for_type === "asset" ? "1635" : "4105")}
            onChange={(e) => setFormData({ ...formData, debit_account_code: e.target.value })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          >
            {formData.for_type === "asset" ? (
              <optgroup label="16xx - Activos Fijos">
                <option value="1605">1605 - Terrenos</option>
                <option value="1610">1610 - Edificios</option>
                <option value="1615">1615 - Muebles</option>
                <option value="1620">1620 - Equipo Oficina</option>
                <option value="1625">1625 - Computación</option>
                <option value="1635">1635 - Maquinaria</option>
                <option value="1640">1640 - Vehículos</option>
              </optgroup>
            ) : (
              <optgroup label="4xxx - Gastos">
                <option value="4105">4105 - Gastos Generales</option>
                <option value="4205">4205 - Servicios</option>
                <option value="4305">4305 - Sueldos</option>
                <option value="5105">5105 - Costos de Ventas</option>
              </optgroup>
            )}
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Fecha Vencimiento *
          </label>
          <input
            type="date"
            value={formData.due_date || ""}
            onChange={(e) => setFormData({ ...formData, due_date: e.target.value })}
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          />
        </div>
      </div>

      <div>
        <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
          Descripción / Notas
        </label>
        <textarea
          value={formData.description || ""}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          rows={2}
          className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          placeholder="Descripción adicional..."
        />
      </div>

      <div className="rounded-lg bg-orange-50 p-3">
        <p className="text-sm text-orange-700">
          <strong>Contabilidad:</strong> Al registrar se crea automáticamente:
          <br />
          <strong>DEBE</strong>: {formData.for_type === "asset" ? "Activo (16xx)" : "Gasto (4xxx)"} - {formData.debit_account_code}
          <br />
          <strong>HABER</strong>: Pasivo (21xx)
        </p>
      </div>

      <div className="flex justify-end gap-3 pt-4">
        <button
          type="button"
          onClick={onCancel}
          className="rounded-md border border-[var(--color-foreground)]/30 px-4 py-2 text-sm font-medium text-[var(--color-foreground)] hover:bg-[var(--color-foreground)]/5"
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