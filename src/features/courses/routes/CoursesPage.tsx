import { useState } from "react";
import { ArchiveIcon, PencilIcon, RotateCcwIcon, TrashIcon, XIcon } from "lucide-react";
import { useCourses } from "../hooks/useCourses";
import { useGroups } from "../../groups/hooks/useGroups";
import { CourseForm } from "../components/CourseForm";
import { Button } from "../../../shared/ui";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { Modal } from "../../../shared/ui/components/Modal";
import type { Course, CreateCourseInput } from "../../../shared/types/Course";

export default function CoursesPage() {
  const { courses, archivedCourses, isLoading, error, createCourse, updateCourse, archiveCourse, restoreCourse, hardDeleteCourse, refetch } = useCourses();
  const { groups } = useGroups();
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingCourse, setEditingCourse] = useState<Course | null>(null);
  const [selectedCourse, setSelectedCourse] = useState<Course | null>(null);
  const [showArchived, setShowArchived] = useState(false);

  const handleCreate = () => {
    setEditingCourse(null);
    setIsFormOpen(true);
  };

  const handleEdit = (course: Course) => {
    setEditingCourse(course);
    setIsFormOpen(true);
    setSelectedCourse(null);
  };

  const handleArchive = async (courseId: string, courseName: string) => {
    if (confirm(`¿Estás seguro de archivar el curso "${courseName}"?`)) {
      const result = await archiveCourse(courseId);
      if (!result.success) {
        alert(result.error || "Error al archivar");
      }
    }
  };

  const handleRestore = async (courseId: string, courseName: string) => {
    if (confirm(`¿Estás seguro de restaurar el curso "${courseName}"?`)) {
      const result = await restoreCourse(courseId);
      if (!result.success) {
        alert(result.error || "Error al restaurar");
      }
    }
  };

  const handleHardDelete = async (courseId: string, courseName: string) => {
    if (confirm(`¿Estás seguro de ELIMINAR PERMANENTEMENTE el curso "${courseName}"? Esta acción no se puede deshacer.`)) {
      const result = await hardDeleteCourse(courseId);
      if (!result.success) {
        alert(result.error || "Error al eliminar");
      }
    }
  };

  const handleSubmit = async (data: CreateCourseInput) => {
    let result;
    if (editingCourse) {
      result = await updateCourse(editingCourse.id, {
        name: data.name,
        description: data.description,
        price: data.price,
        duration: data.duration,
      });
    } else {
      result = await createCourse(data);
    }

    if (result.success) {
      setIsFormOpen(false);
      setEditingCourse(null);
    } else {
      alert(result.error || "Error al guardar");
    }
  };

  // Get groups for selected course
  const courseGroups = selectedCourse 
    ? groups.filter(g => g.courseId === selectedCourse.id)
    : [];

  if (isLoading && courses.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Cursos</h1>
          <button
            onClick={() => setShowArchived(!showArchived)}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
              showArchived 
                ? 'bg-purple-100 text-purple-700' 
                : 'bg-[var(--color-foreground)]/10 text-[var(--color-foreground)] hover:bg-[var(--color-foreground)]/20'
            }`}
          >
            {showArchived ? 'Ver activos' : 'Ver archivados'}
          </button>
        </div>
        {!showArchived && <Button onClick={handleCreate}>Nuevo Curso</Button>}
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
          {error}
          <button onClick={refetch} className="ml-4 underline hover:no-underline">
            Reintentar
          </button>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {courses.map((course) => (
          <div
            key={course.id}
            onClick={() => setSelectedCourse(course)}
            className="bg-[var(--color-background)] rounded-xl shadow-md hover:shadow-lg transition-shadow p-5 cursor-pointer border border-gray-100"
          >
            <div className="flex justify-between items-start mb-3">
              <h3 className="text-lg font-semibold text-[var(--color-foreground)]">{course.name}</h3>
              <div className="flex gap-2">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleEdit(course);
                  }}
                  className="p-2 text-[var(--color-foreground)]/60 hover:text-[var(--color-primary)] hover:bg-[var(--color-primary)]/10 rounded-lg transition-colors"
                >
                  <PencilIcon className="w-4 h-4" />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleArchive(course.id, course.name);
                  }}
                  className="p-2 text-[var(--color-foreground)]/60 hover:text-amber-600 hover:bg-amber-50 rounded-lg transition-colors"
                >
                  <ArchiveIcon className="w-4 h-4" />
                </button>
              </div>
            </div>
            <p className="text-sm text-[var(--color-foreground)]/60 mb-3">{course.code}</p>
            <p className="text-sm text-[var(--color-foreground)]/80 line-clamp-2 mb-4">
              {course.description || "Sin descripción"}
            </p>
            <div className="flex justify-between items-center text-sm">
              <span className="text-[var(--color-foreground)] font-medium">
                ${course.price?.toLocaleString() || 0}
              </span>
              <span className="text-[var(--color-foreground)]/60">{course.duration || 0} horas</span>
            </div>
          </div>
        ))}
      </div>

      {courses.length === 0 && !isLoading && !showArchived && (
        <div className="text-center py-12 text-[var(--color-foreground)]/60">
          No hay cursos registrados. Crea el primero.
        </div>
      )}

      {showArchived && (
        <>
          <h2 className="text-lg font-semibold text-[var(--color-foreground)] mt-8 mb-4">Cursos Archivados</h2>
          {archivedCourses.length === 0 ? (
            <div className="text-center py-8 text-[var(--color-foreground)]/60 bg-[var(--color-foreground)]/5 rounded-lg">
              No hay cursos archivados.
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {archivedCourses.map((course) => (
                <div
                  key={course.id}
                  onClick={() => setSelectedCourse(course)}
                  className="bg-[var(--color-foreground)]/10 rounded-xl p-5 border border-[var(--color-foreground)]/20 opacity-75"
                >
                  <div className="flex justify-between items-start mb-3">
                    <h3 className="text-lg font-semibold text-[var(--color-foreground)]">{course.name}</h3>
                    <div className="flex gap-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRestore(course.id, course.name);
                        }}
                        className="p-2 text-[var(--color-foreground)]/60 hover:text-green-600 hover:bg-green-50 rounded-lg transition-colors"
                        title="Restaurar"
                      >
                        <RotateCcwIcon className="w-4 h-4" />
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleHardDelete(course.id, course.name);
                        }}
                        className="p-2 text-[var(--color-foreground)]/60 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                        title="Eliminar permanentemente"
                      >
                        <TrashIcon className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                  <p className="text-sm text-[var(--color-foreground)]/60 mb-3">{course.code}</p>
                  <p className="text-sm text-[var(--color-foreground)]/80 line-clamp-2 mb-4">
                    {course.description || "Sin descripción"}
                  </p>
                  <div className="flex justify-between items-center text-sm">
                    <span className="text-[var(--color-foreground)] font-medium">
                      ${course.price?.toLocaleString() || 0}
                    </span>
                    <span className="text-[var(--color-foreground)]/60">{course.duration || 0} horas</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </>
      )}

      {/* Course Detail Modal */}
      {selectedCourse && (
        <Modal
          isOpen={true}
          onClose={() => setSelectedCourse(null)}
          title={selectedCourse.name}
        >
          <div className="space-y-6">
            {/* Course Info Cards */}
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-[var(--color-foreground)]/5 p-3 rounded-lg">
                <p className="text-xs text-[var(--color-foreground)]/60 mb-1">Código</p>
                <p className="font-semibold text-sm">{selectedCourse.code}</p>
              </div>
              <div className="bg-[var(--color-foreground)]/5 p-3 rounded-lg">
                <p className="text-xs text-[var(--color-foreground)]/60 mb-1">Precio</p>
                <p className="font-semibold text-sm">${selectedCourse.price?.toLocaleString()}</p>
              </div>
              <div className="bg-[var(--color-foreground)]/5 p-3 rounded-lg">
                <p className="text-xs text-[var(--color-foreground)]/60 mb-1">Duración</p>
                <p className="font-semibold text-sm">{selectedCourse.duration} horas</p>
              </div>
              <div className="bg-[var(--color-foreground)]/5 p-3 rounded-lg">
                <p className="text-xs text-[var(--color-foreground)]/60 mb-1">Grupos</p>
                <p className="font-semibold text-sm">{courseGroups.length}</p>
              </div>
            </div>

            {/* Description */}
            {selectedCourse.description && (
              <div>
                <h4 className="text-sm font-medium text-[var(--color-foreground)] mb-2">Descripción</h4>
                <p className="text-sm text-[var(--color-foreground)]/80">{selectedCourse.description}</p>
              </div>
            )}

            {/* Groups Section */}
            <div>
              <h4 className="text-sm font-medium text-[var(--color-foreground)] mb-3">Grupos del Curso</h4>
              {courseGroups.length > 0 ? (
                <div className="grid grid-cols-1 gap-3">
                  {courseGroups.map((group) => (
                    <div
                      key={group.id}
                      className="bg-[var(--color-foreground)]/5 p-4 rounded-lg border border-[var(--color-foreground)]/20"
                    >
                      <div className="flex justify-between items-start">
                        <div>
                          <h5 className="font-medium text-sm">{group.name}</h5>
                          <p className="text-xs text-[var(--color-foreground)]/60 mt-1">
                            {group.schedule || "Horario no definido"}
                          </p>
                        </div>
                      </div>
                      <div className="mt-2 flex items-center gap-2">
                        <span className={`text-xs px-2 py-1 rounded ${
                          (group.currentStudents || 0) >= (group.maxStudents || 20)
                            ? 'bg-red-100 text-red-700'
                            : 'bg-green-100 text-green-700'
                        }`}>
                          {(group.currentStudents || 0)}/{group.maxStudents || 20}
                        </span>
                        <span className={`text-xs px-2 py-1 rounded ${
                          group.status === "open"
                            ? "bg-green-100 text-green-700"
                            : group.status === "completed"
                            ? "bg-[var(--color-foreground)]/10 text-[var(--color-foreground)]"
                            : "bg-red-100 text-red-700"
                        }`}>
                          {group.status === "open" ? "Activo" : group.status === "completed" ? "Completado" : "Cerrado"}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-[var(--color-foreground)]/60 italic">Este curso aún no tiene grupos asignados.</p>
              )}
            </div>

            {/* Actions */}
            <div className="flex justify-end gap-3 pt-4 border-t">
              <Button variant="secondary" onClick={() => setSelectedCourse(null)}>
                Cerrar
              </Button>
              <Button onClick={() => handleEdit(selectedCourse)}>
                Editar Curso
              </Button>
            </div>
          </div>
        </Modal>
      )}

      {/* Create/Edit Form Modal */}
      {isFormOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-[var(--color-background)] rounded-lg shadow-xl w-full max-w-md p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-bold">
                {editingCourse ? "Editar Curso" : "Nuevo Curso"}
              </h2>
              <button
                onClick={() => {
                  setIsFormOpen(false);
                  setEditingCourse(null);
                }}
                className="p-1 hover:bg-[var(--color-foreground)]/10 rounded"
              >
                <XIcon className="w-5 h-5" />
              </button>
            </div>
            <CourseForm
              course={editingCourse || undefined}
              onSubmit={handleSubmit}
              onCancel={() => {
                setIsFormOpen(false);
                setEditingCourse(null);
              }}
              isLoading={isLoading}
            />
          </div>
        </div>
      )}
    </div>
  );
}
