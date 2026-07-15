import { useState } from "react";
import { useAttendanceThreshold } from "../hooks/useAttendanceThreshold";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";

export function AttendanceThresholdSettings() {
  const { threshold, updateThreshold, isLoading } = useAttendanceThreshold();
  const [localValue, setLocalValue] = useState(threshold);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    setIsSaving(true);
    setMessage(null);
    const result = await updateThreshold(localValue);
    if (result.success) {
      setMessage({ type: "success", text: "Umbral actualizado correctamente" });
    } else {
      setMessage({ type: "error", text: result.error || "Error al guardar" });
    }
    setIsSaving(false);
  };

  if (isLoading) {
    return (
      <div className="bg-[var(--color-background)] rounded-lg shadow border p-4">
        <p className="text-[var(--color-foreground)]/60 text-sm">Cargando configuración...</p>
      </div>
    );
  }

  return (
    <div className="bg-[var(--color-background)] rounded-lg shadow border p-4">
      <h3 className="text-sm font-semibold text-[var(--color-foreground)] mb-3">
        Umbral de advertencia de asistencia
      </h3>
      <p className="text-xs text-[var(--color-foreground)]/60 mb-3">
        Los estudiantes con más de esta cantidad de faltas serán marcados con advertencia.
      </p>
      <div className="flex items-center gap-3">
        <Input
          type="number"
          min={1}
          max={30}
          value={localValue}
          onChange={(e) => setLocalValue(Number(e.target.value))}
          className="w-24"
        />
        <Button onClick={handleSave} disabled={isSaving} size="sm">
          {isSaving ? "Guardando..." : "Guardar"}
        </Button>
      </div>
      {message && (
        <p
          className={`mt-2 text-xs ${
            message.type === "success" ? "text-green-600" : "text-red-600"
          }`}
        >
          {message.text}
        </p>
      )}
    </div>
  );
}
