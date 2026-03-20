import { useState } from "react";
import { Button, Input, Card } from "../../../shared/ui";
import type { Course, CreateCourseInput } from "../../../shared/types/Course";

interface CourseFormProps {
  course?: Course;
  onSubmit: (data: CreateCourseInput) => void;
  onCancel: () => void;
  isLoading?: boolean;
}

export function CourseForm({ course, onSubmit, onCancel, isLoading }: CourseFormProps) {
  const [formData, setFormData] = useState<CreateCourseInput>({
    name: course?.name || "",
    description: course?.description || "",
    code: course?.code || "",
    price: course?.price || 0,
    duration: course?.duration || 0,
  });

  const handleChange = (field: keyof CreateCourseInput, value: string | number) => {
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
          label="Nombre del curso"
          value={formData.name}
          onChange={(e) => handleChange("name", e.target.value)}
          required
        />
        <Input
          label="Código"
          value={formData.code}
          onChange={(e) => handleChange("code", e.target.value)}
          required
        />
        <Input
          label="Descripción"
          value={formData.description || ""}
          onChange={(e) => handleChange("description", e.target.value)}
        />
        <div className="grid grid-cols-2 gap-4">
          <Input
            label="Precio"
            type="number"
            value={formData.price}
            onChange={(e) => handleChange("price", Number(e.target.value))}
            required
          />
          <Input
            label="Duración (horas)"
            type="number"
            value={formData.duration}
            onChange={(e) => handleChange("duration", Number(e.target.value))}
            required
          />
        </div>
        <div className="flex justify-end gap-3 pt-4">
          <Button type="button" variant="secondary" onClick={onCancel}>Cancelar</Button>
          <Button type="submit" loading={isLoading}>{course ? "Actualizar" : "Crear"} curso</Button>
        </div>
      </form>
    </Card>
  );
}
