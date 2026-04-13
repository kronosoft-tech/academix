import { Table, Button } from "../../../shared/ui";
import type { Student } from "../../../shared/types/Student";
import { formatDate } from "../../../shared/utils/formatDate";

interface StudentListProps {
  students: Student[];
  onEdit?: (student: Student) => void;
  onDelete?: (student: Student) => void;
}

export function StudentList({ students, onEdit, onDelete }: StudentListProps) {
  const columns = [
    {
      key: "name",
      header: "Nombre",
    },
    {
      key: "documentNumber",
      header: "Documento",
      render: (student: Student) => (
        <span className="font-mono">{student.documentNumber}</span>
      ),
    },
    {
      key: "email",
      header: "Correo",
    },
    {
      key: "phone",
      header: "Teléfono",
      render: (student: Student) => student.phone || "-",
    },
    {
      key: "createdAt",
      header: "Fecha registro",
      render: (student: Student) => formatDate(student.createdAt),
    },
    {
      key: "actions",
      header: "Acciones",
      className: "text-right",
      render: (student: Student) => (
        <div className="flex justify-end gap-2">
          {onEdit && (
            <Button size="sm" variant="ghost" onClick={() => onEdit(student)}>
              Editar
            </Button>
          )}
          {onDelete && (
            <Button size="sm" variant="ghost" onClick={() => onDelete(student)}>
              Eliminar
            </Button>
          )}
        </div>
      ),
    },
  ];

  return <Table data={students} columns={columns} emptyMessage="No hay estudiantes registrados" />;
}
