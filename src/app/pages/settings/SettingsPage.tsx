// Settings page with compact color pickers

import { ColorPicker } from "../../components/ColorPicker";

export default function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Configuración</h1>
        <p className="mt-1 text-sm text-[var(--color-foreground)]/60">
          Personaliza los colores de la aplicación
        </p>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        <ColorPicker
          label="Color Primario"
          colorKey="primary"
          description="Navegación y acciones principales"
        />
        <ColorPicker
          label="Color Secundario"
          colorKey="secondary"
          description="Elementos de soporte"
        />
        <ColorPicker
          label="Color Acento"
          colorKey="tertiary"
          description="Indicadores y destacados"
        />
        <ColorPicker
          label="Fondo"
          colorKey="background"
          description="Color de fondo de la app"
        />
        <ColorPicker
          label="Texto"
          colorKey="foreground"
          description="Color del texto principal"
        />
      </div>
    </div>
  );
}