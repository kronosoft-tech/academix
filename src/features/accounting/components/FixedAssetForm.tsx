// FixedAssetForm - Form to register fixed assets (activos fijos)

import { useState } from "react";
import { cn } from "../../../lib/utils";

interface FixedAssetRequest {
  name: string;
  asset_type: string;
  description?: string;
  acquisition_date: string;
  acquisition_cost: number;
  useful_life_years: number;
  account_code?: string;  // 16xx (Activos Fijos)
  payment_account_code?: string; // 1105 (caja), 1110 (bancos)
}

interface Props {
  onSubmit: (data: FixedAssetRequest) => Promise<void>;
  onCancel: () => void;
}

export function FixedAssetForm({ onSubmit, onCancel }: Props) {
  const [formData, setFormData] = useState<Partial<FixedAssetRequest>>({
    asset_type: "equipment",
    useful_life_years: 5,
  });
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.name || !formData.acquisition_date || !formData.acquisition_cost) {
      alert("Completa los campos requeridos");
      return;
    }

    setSubmitting(true);
    try {
      await onSubmit(formData as FixedAssetRequest);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <h3 className="text-lg font-semibold text-[var(--color-foreground)]">Registrar Activo Fijo</h3>
      
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Nombre del Activo *
          </label>
          <input
            type="text"
            value={formData.name || ""}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
            placeholder="Ej: Laptop Dell, Escritorio, Camioneta"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Tipo de Activo *
          </label>
          <select
            value={formData.asset_type || "equipment"}
            onChange={(e) => setFormData({ ...formData, asset_type: e.target.value })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          >
            <optgroup label="16xx - Propiedad, Planta y Equipo">
              <option value="land">Terreno (1605)</option>
              <option value="building">Edificio (1610)</option>
              <option value="furniture">Mobiliario (1615)</option>
              <option value="equipment">Equipo de Oficina (1620)</option>
              <option value="computer">Computadora (1625)</option>
              <option value="communication">Equipo de Comunicación (1630)</option>
              <option value="machinery">Maquinaria (1635)</option>
              <option value="vehicle">Vehículo (1640)</option>
              <option value="tools">Herramientas (1645)</option>
              <option value="audio">Equipo de Sonido (1650)</option>
              <option value="video">Equipo de Video (1655)</option>
              <option value="ac">Aire Acondicionado (1660)</option>
              <option value="kitchen">Muebles de Cocina (1665)</option>
              <option value="security">Equipo de Seguridad (1670)</option>
              <option value="gym">Equipo de Gimnasio (1675)</option>
              <option value="musical">Instrumentos Musicales (1680)</option>
              <option value="lab">Equipo de Laboratorio (1685)</option>
              <option value="other">Otro (1690)</option>
            </optgroup>
            <optgroup label="15xx - Intangibles">
              <option value="goodwill">Plusvalía (1505)</option>
              <option value="brand">Marca (1510)</option>
              <option value="patent">Patente (1515)</option>
              <option value="software">Licencia/Software (1520)</option>
            </optgroup>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Fecha de Adquisición *
          </label>
          <input
            type="date"
            value={formData.acquisition_date || ""}
            onChange={(e) => setFormData({ ...formData, acquisition_date: e.target.value })}
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Costo de Adquisición (S/) *
          </label>
          <input
            type="number"
            step="0.01"
            min="0"
            value={formData.acquisition_cost || ""}
            onChange={(e) => setFormData({ ...formData, acquisition_cost: parseFloat(e.target.value) })}
            required
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Cuenta de Pago
          </label>
          <select
            value={formData.payment_account_code || "1105"}
            onChange={(e) => setFormData({ ...formData, payment_account_code: e.target.value })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          >
            <option value="1105">1105 - Caja</option>
            <option value="1110">1110 - Bancos</option>
          </select>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-[var(--color-foreground)]">
            Cuenta Contable (Activo) *
          </label>
          <select
            value={formData.account_code || "1635"}
            onChange={(e) => setFormData({ ...formData, account_code: e.target.value })}
            className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
          >
            <optgroup label="16xx - Propiedad, Planta y Equipo">
              <option value="1605">1605 - Terrenos</option>
              <option value="1610">1610 - Edificios</option>
              <option value="1615">1615 - Muebles y Enseres</option>
              <option value="1620">1620 - Equipo de Oficina</option>
              <option value="1625">1625 - Equipos de Computación</option>
              <option value="1630">1630 - Equipos de Comunicación</option>
              <option value="1635">1635 - Maquinaria y Equipo</option>
              <option value="1640">1640 - Vehículos</option>
              <option value="1645">1645 - Herramientas</option>
              <option value="1650">1650 - Equipos de Sonido</option>
              <option value="1655">1655 - Equipos de Video</option>
              <option value="1660">1660 - Aire Acondicionado</option>
              <option value="1665">1665 - Muebles de Cocina</option>
              <option value="1670">1670 - Equipos de Seguridad</option>
              <option value="1675">1675 - Equipos de Gimnasio</option>
              <option value="1680">1680 - Instrumentos Musicales</option>
              <option value="1685">1685 - Equipos de Laboratorio</option>
              <option value="1690">1690 - Otros Activos Fijos</option>
            </optgroup>
            <optgroup label="15xx - Intangibles">
              <option value="1505">1505 - Plusvalía (Goodwill)</option>
              <option value="1510">1510 - Marcas</option>
              <option value="1515">1515 - Patentes y Derechos</option>
              <option value="1520">1520 - Licencias y Software</option>
            </optgroup>
          </select>
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
          placeholder="Detalles adicionales..."
        />
      </div>

      <div className="rounded-lg bg-[var(--color-primary)]/10 p-3">
        <p className="text-sm text-[var(--color-primary)]">
          <strong>Nota:</strong> El activo fijo seRegistrar contablemente:
          <br />
          <strong>DEBE</strong>: Cuenta de Activo Fijo (16xx) - Ej: 1615 Muebles
          <br />
          <strong>HABER</strong>: Caja/Bancos o Proveedores
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
            "rounded-md bg-[var(--color-primary)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--color-primary)]",
            submitting && "cursor-not-allowed opacity-50"
          )}
        >
          {submitting ? "Guardando..." : "Registrar Activo Fijo"}
        </button>
      </div>
    </form>
  );
}