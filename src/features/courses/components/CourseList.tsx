import { Table, Button } from "../../../shared/ui";
import type { Course } from "../../../shared/types/Course";
import { formatCurrency } from "../../../shared/utils/formatCurrency";

interface CourseListProps {
  courses: Course[];
  onEdit?: (course: Course) => void;
  onDelete?: (course: Course) => void;
}

export function CourseList({ courses, onEdit, onDelete }: CourseListProps) {
  const columns = [
    { key: "code", header: "Código" },
    { key: "name", header: "Nombre" },
    { key: "description", header: "Descripción", render: (c: Course) => c.description || "-" },
    {
      key: "price",
      header: "Precio",
      render: (c: Course) => formatCurrency(c.price),
    },
    { key: "duration", header: "Duración", render: (c: Course) => `${c.duration} horas` },
    {
      key: "actions",
      header: "Acciones",
      className: "text-right",
      render: (course: Course) => (
        <div className="flex justify-end gap-2">
          {onEdit && <Button size="sm" variant="ghost" onClick={() => onEdit(course)}>Editar</Button>}
          {onDelete && <Button size="sm" variant="ghost" onClick={() => onDelete(course)}>Eliminar</Button>}
        </div>
      ),
    },
  ];

  return <Table data={courses} columns={columns} emptyMessage="No hay cursos registrados" />;
}
