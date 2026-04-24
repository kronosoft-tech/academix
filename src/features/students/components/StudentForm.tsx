import { useState } from "react";
import { Button, Input, Select, Card } from "../../../shared/ui";
import type { Student, CreateStudentInput } from "../../../shared/types/Student";

interface StudentFormProps {
  student?: Student;
  onSubmit: (data: CreateStudentInput) => void;
  onCancel: () => void;
  isLoading?: boolean;
}

const documentTypes = [
  { value: "cc", label: "Cédula de ciudadanía" },
  { value: "ti", label: "Tarjeta de identidad" },
  { value: "ce", label: "Cédula de extrangería" },
  { value: "rc", label: "Registro civil" },
  { value: "nip", label: "NIP" },
];

export function StudentForm({ student, onSubmit, onCancel, isLoading }: StudentFormProps) {
  const [formData, setFormData] = useState<CreateStudentInput>({
    name: student?.name || "",
    documentNumber: student?.documentNumber || "",
    documentType: student?.documentType || "cc",
    email: student?.email || "",
    phone: student?.phone || "",
    address: student?.address || "",
    birthDate: student?.birthDate || "",
    guardianName: student?.guardianName || "",
    guardianDocument: student?.guardianDocument || "",
    guardianPhone: student?.guardianPhone || "",
  });

  const handleChange = (field: keyof CreateStudentInput, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(formData);
  };

  return (
    <Card>
      <form onSubmit={handleSubmit} className="space-y-4">
        <Input
          label="Nombre completo"
          value={formData.name}
          onChange={(e) => handleChange("name", e.target.value)}
          required
        />
        <div className="grid grid-cols-2 gap-4">
          <Select
            label="Tipo de documento"
            value={formData.documentType}
            onChange={(e) => handleChange("documentType", e.target.value)}
            options={documentTypes}
          />
          <Input
            label="Número de documento"
            value={formData.documentNumber}
            onChange={(e) => handleChange("documentNumber", e.target.value)}
            required
          />
        </div>
        <Input
          label="Correo electrónico"
          type="email"
          value={formData.email}
          onChange={(e) => handleChange("email", e.target.value)}
          required
        />
        <div className="grid grid-cols-2 gap-4">
          <Input
            label="Teléfono"
            type="tel"
            value={formData.phone || ""}
            onChange={(e) => handleChange("phone", e.target.value)}
          />
          <Input
            label="Dirección"
            value={formData.address || ""}
            onChange={(e) => handleChange("address", e.target.value)}
          />
        </div>
        <Input
          label="Fecha de nacimiento"
          type="date"
          value={formData.birthDate || ""}
          onChange={(e) => handleChange("birthDate", e.target.value)}
        />
        <div className="border-t pt-4 mt-4">
          <h4 className="text-sm font-medium text-[var(--color-foreground)] mb-3">Información del acudiente</h4>
          <div className="grid grid-cols-2 gap-4">
            <Input
              label="Nombre del acudiente"
              value={formData.guardianName || ""}
              onChange={(e) => handleChange("guardianName", e.target.value)}
            />
            <Input
              label="Documento del acudiente"
              value={formData.guardianDocument || ""}
              onChange={(e) => handleChange("guardianDocument", e.target.value)}
            />
          </div>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <Input
              label="Teléfono del acudiente"
              type="tel"
              value={formData.guardianPhone || ""}
              onChange={(e) => handleChange("guardianPhone", e.target.value)}
            />
          </div>
        </div>
        <div className="flex justify-end gap-3 pt-4">
          <Button type="button" variant="secondary" onClick={onCancel}>
            Cancelar
          </Button>
          <Button type="submit" loading={isLoading}>
            {student ? "Actualizar" : "Crear"} estudiante
          </Button>
        </div>
      </form>
    </Card>
  );
}
