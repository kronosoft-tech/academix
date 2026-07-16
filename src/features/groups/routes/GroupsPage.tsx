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
    startDate: "",
    maxStudents: 20,
    classDuration: 60,
    skippedDates: [] as string[],
  });
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [newSkippedDate, setNewSkippedDate] = useState("");

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
      startDate: group.startDate || "",
      maxStudents: group.maxStudents,
      classDuration: group.classDuration || 60,
      skippedDates: group.skippedDates || [],
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
    if (!formData.startTime || !formData.endTime) {
      setSubmitError("Debes definir el horario de clase");
      return;
    }

    // Calcular duración automáticamente del horario
    const [startH, startM] = formData.startTime.split(":").map(Number);
    const [endH, endM] = formData.endTime.split(":").map(Number);
    const startMinutes = startH * 60 + startM;
    const endMinutes = endH * 60 + endM;
    const calculatedDuration = endMinutes - startMinutes;

    if (calculatedDuration <= 0) {
      setSubmitError("La hora de fin debe ser posterior a la hora de inicio");
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
        startDate: formData.startDate || undefined,
        maxStudents: formData.maxStudents,
        classDuration: calculatedDuration,
        skippedDates: formData.skippedDates.length > 0 ? formData.skippedDates : undefined,
      });

      if (result.success) {
        setShowForm(false);
        setEditingGroup(null);
        setFormData({ name: "", courseId: "", professorId: "", days: [], startTime: "", endTime: "", startDate: "", maxStudents: 20, classDuration: 60, skippedDates: [] });
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
        startDate: formData.startDate || undefined,
        maxStudents: formData.maxStudents,
        classDuration: calculatedDuration,
        skippedDates: formData.skippedDates.length > 0 ? formData.skippedDates : undefined,
      });

      if (result.success) {
        setShowForm(false);
        setFormData({ name: "", courseId: "", professorId: "", days: [], startTime: "", endTime: "", startDate: "", maxStudents: 20, classDuration: 60, skippedDates: [] });
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
              <div>
                <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
                  Hora de inicio
                </label>
                <div className="flex gap-2">
                  <select
                    value={formData.startTime.split(":")[0] ? (() => {
                      const h = parseInt(formData.startTime.split(":")[0] || "8");
                      return h === 0 ? "12" : h > 12 ? String(h - 12) : String(h);
                    })() : "8"}
                    onChange={(e) => {
                      const h = parseInt(e.target.value);
                      const isPM = formData.startTime.includes("PM") || (formData.startTime.split(":")[0] && parseInt(formData.startTime.split(":")[0]) >= 12);
                      const fullHour = isPM ? (h === 12 ? 12 : h + 12) : (h === 12 ? 0 : h);
                      const min = formData.startTime.split(":")[1] || "00";
                      setFormData({ ...formData, startTime: `${String(fullHour).padStart(2, "0")}:${min}` });
                    }}
                    className="flex-1 px-2 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    {[1,2,3,4,5,6,7,8,9,10,11,12].map(h => (
                      <option key={h} value={h}>{h}</option>
                    ))}
                  </select>
                  <select
                    value={formData.startTime.split(":")[1] || "00"}
                    onChange={(e) => {
                      const h = formData.startTime.split(":")[0] || "08";
                      setFormData({ ...formData, startTime: `${h}:${e.target.value}` });
                    }}
                    className="w-20 px-2 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    {["00","05","10","15","20","25","30","35","40","45","50","55"].map(m => (
                      <option key={m} value={m}>{m}</option>
                    ))}
                  </select>
                  <select
                    value={(() => {
                      const h = parseInt(formData.startTime.split(":")[0] || "8");
                      return h >= 12 ? "PM" : "AM";
                    })()}
                    onChange={(e) => {
                      const parts = formData.startTime.split(":");
                      let h = parseInt(parts[0] || "8");
                      const min = parts[1] || "00";
                      const isPM = e.target.value === "PM";
                      if (isPM && h < 12) h += 12;
                      if (!isPM && h === 12) h = 0;
                      if (isPM && h === 24) h = 12;
                      setFormData({ ...formData, startTime: `${String(h).padStart(2, "0")}:${min}` });
                    }}
                    className="w-16 px-2 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="AM">AM</option>
                    <option value="PM">PM</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
                  Hora de fin
                </label>
                <div className="flex gap-2">
                  <select
                    value={formData.endTime.split(":")[0] ? (() => {
                      const h = parseInt(formData.endTime.split(":")[0] || "10");
                      return h === 0 ? "12" : h > 12 ? String(h - 12) : String(h);
                    })() : "10"}
                    onChange={(e) => {
                      const h = parseInt(e.target.value);
                      const isPM = formData.endTime.includes("PM") || (formData.endTime.split(":")[0] && parseInt(formData.endTime.split(":")[0]) >= 12);
                      const fullHour = isPM ? (h === 12 ? 12 : h + 12) : (h === 12 ? 0 : h);
                      const min = formData.endTime.split(":")[1] || "00";
                      setFormData({ ...formData, endTime: `${String(fullHour).padStart(2, "0")}:${min}` });
                    }}
                    className="flex-1 px-2 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    {[1,2,3,4,5,6,7,8,9,10,11,12].map(h => (
                      <option key={h} value={h}>{h}</option>
                    ))}
                  </select>
                  <select
                    value={formData.endTime.split(":")[1] || "00"}
                    onChange={(e) => {
                      const h = formData.endTime.split(":")[0] || "10";
                      setFormData({ ...formData, endTime: `${h}:${e.target.value}` });
                    }}
                    className="w-20 px-2 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    {["00","05","10","15","20","25","30","35","40","45","50","55"].map(m => (
                      <option key={m} value={m}>{m}</option>
                    ))}
                  </select>
                  <select
                    value={(() => {
                      const h = parseInt(formData.endTime.split(":")[0] || "10");
                      return h >= 12 ? "PM" : "AM";
                    })()}
                    onChange={(e) => {
                      const parts = formData.endTime.split(":");
                      let h = parseInt(parts[0] || "10");
                      const min = parts[1] || "00";
                      const isPM = e.target.value === "PM";
                      if (isPM && h < 12) h += 12;
                      if (!isPM && h === 12) h = 0;
                      if (isPM && h === 24) h = 12;
                      setFormData({ ...formData, endTime: `${String(h).padStart(2, "0")}:${min}` });
                    }}
                    className="w-16 px-2 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="AM">AM</option>
                    <option value="PM">PM</option>
                  </select>
                </div>
              </div>
            </div>

            {/* Duración calculada */}
            {formData.startTime && formData.endTime && (() => {
              const [startH, startM] = formData.startTime.split(":").map(Number);
              const [endH, endM] = formData.endTime.split(":").map(Number);
              const startMinutes = startH * 60 + startM;
              const endMinutes = endH * 60 + endM;
              const duration = endMinutes - startMinutes;
              if (duration > 0) {
                return (
                  <div className="bg-blue-50 border border-blue-200 rounded-lg px-4 py-2">
                    <span className="text-sm text-blue-800">
                      <strong>Duración por clase:</strong> {duration} minutos ({Math.floor(duration / 60)}h {duration % 60 > 0 ? `${duration % 60}min` : ""})
                    </span>
                  </div>
                );
              }
              return null;
            })()}

            {/* Fecha de inicio */}
            <div>
              <Input
                label="Fecha de inicio"
                type="date"
                value={formData.startDate}
                onChange={(e) => setFormData({ ...formData, startDate: e.target.value })}
              />
            </div>

            {/* Fechas omitidas */}
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-2">
                Fechas omitidas (feriados, vacaciones, etc.)
              </label>
              <div className="flex gap-2 mb-2">
                <Input
                  type="date"
                  value={newSkippedDate}
                  onChange={(e) => setNewSkippedDate(e.target.value)}
                />
                <Button
                  type="button"
                  variant="secondary"
                  onClick={() => {
                    if (newSkippedDate && !formData.skippedDates.includes(newSkippedDate)) {
                      setFormData({
                        ...formData,
                        skippedDates: [...formData.skippedDates, newSkippedDate].sort(),
                      });
                      setNewSkippedDate("");
                    }
                  }}
                >
                  Agregar
                </Button>
              </div>
              {formData.skippedDates.length > 0 && (
                <div className="flex flex-wrap gap-2">
                  {formData.skippedDates.map((date) => (
                    <span
                      key={date}
                      className="inline-flex items-center gap-1 px-2 py-1 bg-gray-100 rounded text-sm"
                    >
                      {date}
                      <button
                        type="button"
                        onClick={() => {
                          setFormData({
                            ...formData,
                            skippedDates: formData.skippedDates.filter((d) => d !== date),
                          });
                        }}
                        className="text-red-500 hover:text-red-700"
                      >
                        ×
                      </button>
                    </span>
                  ))}
                </div>
              )}
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
                {group.startDate && (
                  <p className="text-sm text-[var(--color-foreground)]/60 mt-1">
                    Inicio: {group.startDate}
                  </p>
                )}
                {group.classDuration && (
                  <p className="text-sm text-[var(--color-foreground)]/60 mt-1">
                    Duración: {group.classDuration} min
                  </p>
                )}
                {group.calculatedEndDate && (
                  <p className="text-sm text-[var(--color-foreground)]/60 mt-1">
                    Fin calculado: {group.calculatedEndDate}
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
