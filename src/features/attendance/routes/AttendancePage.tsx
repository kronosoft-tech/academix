import { useState } from "react";
import { useAttendance } from "../hooks/useAttendance";
import { useStudents } from "../../students/hooks/useStudents";
import { useGroups } from "../../groups/hooks/useGroups";
import { Card } from "../../../shared/ui/components/Card";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { SearchableSelect } from "../../../shared/ui/components/SearchableSelect";

export default function AttendancePage() {
  const { attendance, isLoading, error, createAttendance, refetch } = useAttendance();
  const { students } = useStudents();
  const { groups } = useGroups();
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    studentId: "",
    groupId: "",
    date: new Date().toISOString().split("T")[0],
    status: "present" as "present" | "absent" | "late" | "excused",
    notes: "",
  });
  const [submitError, setSubmitError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError(null);

    if (!formData.studentId) {
      setSubmitError("Debes seleccionar un estudiante");
      return;
    }
    if (!formData.groupId) {
      setSubmitError("Debes seleccionar un grupo");
      return;
    }

    const result = await createAttendance({
      studentId: formData.studentId,
      groupId: formData.groupId,
      date: formData.date,
      status: formData.status,
      notes: formData.notes || undefined,
    });

    if (result.success) {
      setShowForm(false);
      setFormData({
        studentId: "",
        groupId: "",
        date: new Date().toISOString().split("T")[0],
        status: "present",
        notes: "",
      });
    } else {
      setSubmitError(result.error || "Error al registrar asistencia");
    }
  };

  if (isLoading && attendance.length === 0) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Asistencia</h1>
        <div className="flex gap-2">
          <Button onClick={() => setShowForm(!showForm)}>
            {showForm ? "Cancelar" : "Nueva Asistencia"}
          </Button>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {submitError && (
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
          {submitError}
        </div>
      )}

      {showForm && (
        <Card className="mb-6">
          <h2 className="text-lg font-semibold mb-4">Registrar Asistencia</h2>
          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Buscador de Estudiante */}
            <SearchableSelect
              label="Estudiante"
              required
              placeholder="Buscar por nombre, apellido o ID..."
              value={formData.studentId}
              onChange={(id) => setFormData({ ...formData, studentId: id })}
              options={students}
              searchFields={["name", "id", "documentNumber"] as (keyof typeof students[0])[]}
              displayFormatter={(student) => `${student.name} - ${student.documentNumber}`}
              getItemValue={(student) => student.id}
              notFoundMessage="No se encontraron estudiantes"
            />

            {/* Buscador de Grupo */}
            <SearchableSelect
              label="Grupo"
              required
              placeholder="Buscar grupo..."
              value={formData.groupId}
              onChange={(id) => setFormData({ ...formData, groupId: id })}
              options={groups}
              searchFields={["name", "id"] as (keyof typeof groups[0])[]}
              displayFormatter={(group) => group.name}
              getItemValue={(group) => group.id}
              notFoundMessage="No se encontraron grupos"
            />

            <Input
              label="Fecha"
              type="date"
              value={formData.date}
              onChange={(e) => setFormData({ ...formData, date: e.target.value })}
              required
            />

            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
                Estado
              </label>
              <select
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                value={formData.status}
                onChange={(e) => setFormData({ ...formData, status: e.target.value as typeof formData.status })}
              >
                <option value="present">Presente</option>
                <option value="absent">Ausente</option>
                <option value="late">Tarde</option>
                <option value="excused">Justificado</option>
              </select>
            </div>

            <Input
              label="Notas"
              placeholder="Observaciones..."
              value={formData.notes}
              onChange={(e) => setFormData({ ...formData, notes: e.target.value })}
            />

            <div className="flex gap-2">
              <Button type="submit" loading={isLoading}>Registrar</Button>
              <Button type="button" variant="secondary" onClick={() => setShowForm(false)}>
                Cancelar
              </Button>
            </div>
          </form>
        </Card>
      )}

      {attendance.length === 0 ? (
        <Card className="text-center py-12">
          <p className="text-[var(--color-foreground)]/60">No hay registros de asistencia</p>
          <Button className="mt-4" onClick={() => setShowForm(true)}>
            Registrar Primera Asistencia
          </Button>
        </Card>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-[var(--color-foreground)]/5">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Fecha
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Estudiante
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Grupo
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Estado
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Notas
                </th>
              </tr>
            </thead>
            <tbody className="bg-[var(--color-background)] divide-y divide-gray-200">
              {attendance.map((record) => (
                <tr key={record.id} className="hover:bg-[var(--color-foreground)]/5">
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]">
                    {record.date}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]">
                    {record.studentId.substring(0, 8)}...
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]">
                    {record.groupId.substring(0, 8)}...
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        record.status === "present"
                          ? "bg-green-100 text-green-800"
                          : record.status === "absent"
                          ? "bg-red-100 text-red-800"
                          : record.status === "late"
                          ? "bg-yellow-100 text-yellow-800"
                          : "bg-[var(--color-foreground)]/10 text-[var(--color-foreground)]"
                      }`}
                    >
                      {record.status === "present"
                        ? "Presente"
                        : record.status === "absent"
                        ? "Ausente"
                        : record.status === "late"
                        ? "Tarde"
                        : "Justificado"}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]/60">
                    {record.notes || "-"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="mt-4">
        <Button variant="secondary" onClick={refetch}>
          Actualizar
        </Button>
      </div>
    </div>
  );
}
