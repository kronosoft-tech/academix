import { useState } from "react";
import { useGroups } from "../hooks/useGroups";
import { useCourses } from "../../courses/hooks/useCourses";
import { useUsers } from "../../users/hooks/useUsers";
import { useStudents } from "../../students/hooks/useStudents";
import { Card } from "../../../shared/ui/components/Card";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { PencilIcon, TrashIcon } from "lucide-react";
import GroupDetailInline from "../components/GroupDetailInline";
import type { Group } from "../../../shared/types/Group";

export default function GroupsPage() {
  const { groups, isLoading, error, createGroup, updateGroup, deleteGroup, refetch } = useGroups();
  const { courses } = useCourses();
  const { users } = useUsers();
  const { students } = useStudents();
  const [showForm, setShowForm] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedGroupId, setSelectedGroupId] = useState<string | null>(null);
  const [editingGroup, setEditingGroup] = useState<Group | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    courseId: "",
    professorId: "",
    days: [] as string[],
    startTime: "",
    endTime: "",
    maxStudents: 20,
  });
  const [submitError, setSubmitError] = useState<string | null>(null);

  // Filtrar solo usuarios con rol de profesor o gerente (comparar en lowercase)
  const professors = users.filter(u => 
    u.role === "profesor" || u.role === "gerente" || u.role === "admin"
  );

  const filteredGroups = groups.filter((group) =>
    group.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleEditGroup = (group: Group) => {
    setEditingGroup(group);
    setShowForm(true);
    setFormData({
      name: group.name,
      courseId: group.courseId,
      professorId: group.professorId || "",
      days: group.days || [],
      startTime: group.startTime || "",
      endTime: group.endTime || "",
      maxStudents: group.maxStudents,
    });
  };

  const handleDeleteGroup = async (groupId: string, groupName: string) => {
    if (confirm(`¿Eliminar el grupo "${groupName}"?`)) {
      const result = await deleteGroup(groupId);
      if (!result.success) {
        alert(result.error || "Error al eliminar");
      }
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError(null);

    // Validaciones
    if (!formData.courseId) {
      setSubmitError("Debes seleccionar un curso");
      return;
    }
    if (!formData.professorId) {
      setSubmitError("Debes seleccionar un profesor");
      return;
    }

    if (editingGroup) {
      // Update existing group
      const result = await updateGroup(editingGroup.id, {
        name: formData.name,
        professorId: formData.professorId || undefined,
        days: formData.days,
        startTime: formData.startTime,
        endTime: formData.endTime,
        maxStudents: formData.maxStudents,
      });

      if (result.success) {
        setShowForm(false);
        setEditingGroup(null);
        setFormData({ name: "", courseId: "", professorId: "", days: [], startTime: "", endTime: "", maxStudents: 20 });
      } else {
        setSubmitError(result.error || "Error al actualizar grupo");
      }
    } else {
      // Create new group
      const result = await createGroup({
        name: formData.name,
        courseId: formData.courseId,
        professorId: formData.professorId,
        days: formData.days,
        startTime: formData.startTime,
        endTime: formData.endTime,
        maxStudents: formData.maxStudents,
      });

      if (result.success) {
        setShowForm(false);
        setFormData({ name: "", courseId: "", professorId: "", days: [], startTime: "", endTime: "", maxStudents: 20 });
      } else {
        setSubmitError(result.error || "Error al crear grupo");
      }
    }
  };

  if (isLoading && groups.length === 0) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spinner size="lg" />
      </div>
    );
  }

  if (selectedGroupId) {
    return (
      <GroupDetailInline 
        groupId={selectedGroupId} 
        groups={groups}
        courses={courses}
        users={users}
        students={students}
        onBack={() => setSelectedGroupId(null)}
        onEdit={(group) => {
          handleEditGroup(group);
          setSelectedGroupId(null);
        }}
      />
    );
  }

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Grupos</h1>
        <Button onClick={() => { setShowForm(!showForm); if (showForm) setEditingGroup(null); }}>
          {showForm ? "Cancelar" : "Nuevo Grupo"}
        </Button>
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
          <h2 className="text-lg font-semibold mb-4">{editingGroup ? "Editar Grupo" : "Crear Nuevo Grupo"}</h2>
          <form onSubmit={handleSubmit} className="space-y-4">
            <Input
              label="Nombre del Grupo"
              placeholder="Ej: Grupo A - Inglés Básico"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
            />

            {/* Selector de Curso */}
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
                Curso <span className="text-red-500">*</span>
              </label>
              <select
                value={formData.courseId}
                onChange={(e) => setFormData({ ...formData, courseId: e.target.value })}
                required
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">Selecciona un curso</option>
                {courses.map((course) => (
                  <option key={course.id} value={course.id}>
                    {course.name} ({course.code})
                  </option>
                ))}
              </select>
            </div>

            {/* Selector de Profesor */}
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
                Profesor <span className="text-red-500">*</span>
              </label>
              <select
                value={formData.professorId}
                onChange={(e) => setFormData({ ...formData, professorId: e.target.value })}
                required
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">Selecciona un profesor</option>
                {professors.map((prof) => (
                  <option key={prof.id} value={prof.id}>
                    {prof.name} ({prof.role})
                  </option>
                ))}
              </select>
            </div>

            {/* Días de clase */}
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-2">
                Días de clase
              </label>
              <div className="flex flex-wrap gap-2">
                {["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"].map((day) => (
                  <label key={day} className="flex items-center gap-1.5 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={formData.days.includes(day)}
                      onChange={(e) => {
                        const newDays = e.target.checked
                          ? [...formData.days, day]
                          : formData.days.filter((d) => d !== day);
                        setFormData({ ...formData, days: newDays });
                      }}
                      className="rounded text-[var(--color-primary)]"
                    />
                    <span className="text-sm">{day}</span>
                  </label>
                ))}
              </div>
            </div>

            {/* Horario */}
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Hora de inicio"
                type="time"
                value={formData.startTime}
                onChange={(e) => setFormData({ ...formData, startTime: e.target.value })}
              />
              <Input
                label="Hora de fin"
                type="time"
                value={formData.endTime}
                onChange={(e) => setFormData({ ...formData, endTime: e.target.value })}
              />
            </div>
            
            <Input
              label="Cupo Máximo"
              type="number"
              placeholder="20"
              value={formData.maxStudents}
              onChange={(e) => setFormData({ ...formData, maxStudents: parseInt(e.target.value) || 20 })}
              required
            />
            
            <div className="flex gap-2">
              <Button type="submit" loading={isLoading}>{editingGroup ? "Actualizar Grupo" : "Crear Grupo"}</Button>
              <Button type="button" variant="secondary" onClick={() => { setShowForm(false); setEditingGroup(null); }}>
                Cancelar
              </Button>
            </div>
          </form>
        </Card>
      )}

      <div className="mb-4">
        <Input
          placeholder="Buscar grupos..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      {filteredGroups.length === 0 ? (
        <Card className="text-center py-12">
          <p className="text-[var(--color-foreground)]/60">No hay grupos registrados</p>
          <Button className="mt-4" onClick={() => setShowForm(true)}>
            Crear Primer Grupo
          </Button>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredGroups.map((group) => {
            const isFull = (group.currentStudents ?? 0) >= group.maxStudents;
            return (
              <div 
                key={group.id} 
                className="hover:shadow-md transition-shadow cursor-pointer"
                onClick={() => setSelectedGroupId(group.id)}
              >
                <Card>
                <div className="flex justify-between items-start">
                  <h3 className="font-semibold text-[var(--color-foreground)]">{group.name}</h3>
                  <div className="flex gap-1" onClick={(e) => e.stopPropagation()}>
                    <button
                      onClick={() => handleEditGroup(group)}
                      className="p-1.5 text-[var(--color-foreground)]/60 hover:text-[var(--color-primary)] hover:bg-[var(--color-primary)]/10 rounded"
                    >
                      <PencilIcon className="w-4 h-4" />
                    </button>
                    <button
                      onClick={() => handleDeleteGroup(group.id, group.name)}
                      className="p-1.5 text-[var(--color-foreground)]/60 hover:text-red-600 hover:bg-red-50 rounded"
                    >
                      <TrashIcon className="w-4 h-4" />
                    </button>
                  </div>
                </div>
                <div className="flex items-center gap-2 mt-1">
                  <span
                    className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      isFull
                        ? "bg-red-100 text-red-800"
                        : "bg-green-100 text-green-800"
                    }`}
                  >
                    {group.currentStudents}/{group.maxStudents} estudiantes
                  </span>
                </div>
                {group.days && group.days.length > 0 && (
                  <p className="text-sm text-[var(--color-foreground)]/60 mt-1">
                    {group.days.join(", ")}
                    {group.startTime && ` - ${group.startTime}`}
                    {group.endTime && ` a ${group.endTime}`}
                  </p>
                )}
                <div className="mt-3 flex gap-2">
                  <span
                    className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                      group.status === "open"
                        ? "bg-green-100 text-green-800"
                        : group.status === "completed"
                        ? "bg-[var(--color-foreground)]/10 text-[var(--color-foreground)]"
                        : "bg-red-100 text-red-800"
                    }`}
                  >
                    {group.status === "open" ? "Activo" : group.status === "completed" ? "Completado" : "Cerrado"}
                  </span>
                  {isFull && (
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                      LLENO
                    </span>
                  )}
                </div>
                </Card>
              </div>
            );
          })}
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
