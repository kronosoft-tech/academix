import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeftIcon } from "lucide-react";
import { useCourses } from "../hooks/useCourses";
import { useGroups } from "../../groups/hooks/useGroups";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { Button } from "../../../shared/ui";

export default function CourseDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { courses, isLoading: coursesLoading } = useCourses();
  const { groups, isLoading: groupsLoading } = useGroups();

  const course = courses.find((c) => c.id === id);
  const courseGroups = groups.filter((g) => g.courseId === id);

  if (coursesLoading && !course) {
    return (
      <div className="flex items-center justify-center h-64">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!course) {
    return (
      <div className="p-6">
        <Button variant="secondary" onClick={() => navigate("/courses")} className="mb-4">
          <ArrowLeftIcon className="w-4 h-4 mr-2" />
          Volver a Cursos
        </Button>
        <div className="text-center py-12 text-[var(--color-foreground)]/60">Curso no encontrado</div>
      </div>
    );
  }

  return (
    <div className="p-6">
      <Button variant="secondary" onClick={() => navigate("/courses")} className="mb-6">
        <ArrowLeftIcon className="w-4 h-4 mr-2" />
        Volver a Cursos
      </Button>

      <div className="mb-6">
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">{course.name}</h1>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Código</p>
          <p className="font-semibold">{course.code}</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Precio</p>
          <p className="font-semibold">${course.price?.toLocaleString()}</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Duración</p>
          <p className="font-semibold">{course.duration} horas</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Grupos</p>
          <p className="font-semibold">{courseGroups.length}</p>
        </div>
      </div>

      <div className="bg-[var(--color-background)] p-6 rounded-lg shadow mb-8">
        <h3 className="font-semibold mb-2">Descripción</h3>
        <p className="text-[var(--color-foreground)]/80">{course.description || "Sin descripción"}</p>
      </div>

      <div className="mb-4">
        <h2 className="text-xl font-semibold text-[var(--color-foreground)]">Grupos del Curso</h2>
      </div>

      {groupsLoading ? (
        <div className="flex items-center justify-center h-32">
          <Spinner size="lg" />
        </div>
      ) : courseGroups.length === 0 ? (
        <div className="bg-[var(--color-background)] p-6 rounded-lg shadow text-center text-[var(--color-foreground)]/60">
          No hay grupos registrados para este curso
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {courseGroups.map((group) => (
            <div
              key={group.id}
              className="bg-[var(--color-background)] p-4 rounded-lg shadow border border-gray-100 cursor-pointer hover:shadow-md transition-shadow"
              onClick={() => navigate(`/groups/${group.id}`)}
            >
              <h4 className="font-semibold">{group.name}</h4>
              <p className="text-sm text-[var(--color-foreground)]/60 mt-1">
                Horario: {group.schedule || "No definido"}
              </p>
              <div className="flex justify-between items-center mt-3 text-sm">
                <span className="text-[var(--color-foreground)]/60">
                  {group.currentStudents || 0}/{group.maxStudents || 20} estudiantes
                </span>
                <span
                  className={`px-2 py-1 rounded text-xs ${
                    (group.currentStudents || 0) >= (group.maxStudents || 20)
                      ? "bg-red-100 text-red-700"
                      : "bg-green-100 text-green-700"
                  }`}
                >
                  {(group.currentStudents || 0) >= (group.maxStudents || 20)
                    ? "Lleno"
                    : "Disponible"}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
