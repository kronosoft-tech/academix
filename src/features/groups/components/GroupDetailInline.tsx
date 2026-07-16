import { useState } from "react";
import { ArrowLeftIcon, PencilIcon, BarChart3Icon, ClipboardListIcon } from "lucide-react";
import type { Group } from "../../../shared/types/Group";
import type { Course } from "../../../shared/types/Course";
import type { User } from "../../../shared/types/User";
import type { Student } from "../../../shared/types/Student";
import { Button } from "../../../shared/ui/components/Button";
import GroupAttendanceStatsView from "./GroupAttendanceStatsView";
import DailyAttendanceForm from "./DailyAttendanceForm";
import AtRiskStudentsWidget from "./AtRiskStudentsWidget";
import { useAttendanceThreshold } from "../../students/hooks/useAttendanceThreshold";

interface Props {
  groupId: string;
  groups: Group[];
  courses: Course[];
  users: User[];
  students: Student[];
  onBack: () => void;
  onEdit: (group: Group) => void;
}

type AttendanceView = "stats" | "daily";

export default function GroupDetailInline({
  groupId,
  groups,
  courses,
  users,
  students,
  onBack,
  onEdit,
}: Props) {
  const group = groups.find((g) => g.id === groupId);
  const course = courses.find((c) => c.id === group?.courseId);
  const professor = users.find((u) => u.id === group?.professorId);
  const [attendanceView, setAttendanceView] = useState<AttendanceView>("stats");
  const { threshold } = useAttendanceThreshold();

  const professorName = professor?.name || "No asignado";

  // Get students enrolled in this group
  const groupStudents = students.filter((s) => s.groupId === groupId);

  if (!group) {
    return (
      <div className="p-6">
        <p>Grupo no encontrado</p>
        <Button onClick={onBack}>Volver</Button>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <button onClick={onBack} className="p-2 hover:bg-[var(--color-foreground)]/10 rounded-lg">
            <ArrowLeftIcon className="w-5 h-5" />
          </button>
          <div>
            <h1 className="text-2xl font-bold">{group.name}</h1>
            <p className="text-[var(--color-foreground)]/60">
              {group.days && group.days.length > 0
                ? `${group.days.join(", ")}${group.startTime ? ` - ${group.startTime}` : ""}${
                    group.endTime ? ` a ${group.endTime}` : ""
                  }`
                : "Horario no definido"}
            </p>
          </div>
        </div>
        <Button onClick={() => onEdit(group)}>
          <PencilIcon className="w-4 h-4 mr-2" />
          Editar
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Curso</p>
          <p className="font-semibold">{course?.name || "-"}</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Profesor</p>
          <p className="font-semibold">{professorName}</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Cupo</p>
          <p className="font-semibold">
            {group.currentStudents || 0} / {group.maxStudents}
          </p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
          <p className="text-sm text-[var(--color-foreground)]/60">Estado</p>
          <p className="font-semibold capitalize">
            {group.status === "open"
              ? "Activo"
              : group.status === "completed"
                ? "Completado"
                : "Cerrado"}
          </p>
        </div>
      </div>

      {/* Additional Group Info */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        {group.startDate && (
          <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
            <p className="text-sm text-[var(--color-foreground)]/60">Fecha de Inicio</p>
            <p className="font-semibold">{group.startDate}</p>
          </div>
        )}
        {group.classDuration && (
          <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
            <p className="text-sm text-[var(--color-foreground)]/60">Duración de Clase</p>
            <p className="font-semibold">{group.classDuration} minutos</p>
          </div>
        )}
        {group.calculatedEndDate && (
          <div className="bg-[var(--color-background)] p-4 rounded-lg shadow">
            <p className="text-sm text-[var(--color-foreground)]/60">Fecha de Fin Calculada</p>
            <p className="font-semibold">{group.calculatedEndDate}</p>
          </div>
        )}
      </div>

      {/* Skipped Dates */}
      {group.skippedDates && group.skippedDates.length > 0 && (
        <div className="mb-8">
          <h3 className="text-sm font-medium text-[var(--color-foreground)]/60 mb-2">Fechas Omitidas</h3>
          <div className="flex flex-wrap gap-2">
            {group.skippedDates.map((date) => (
              <span
                key={date}
                className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800"
              >
                {date}
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Attendance Section Tabs */}
      <div className="mb-6">
        <div className="flex gap-2 border-b">
          <button
            onClick={() => setAttendanceView("stats")}
            className={`flex items-center gap-2 px-4 py-2 font-medium transition-colors ${
              attendanceView === "stats"
                ? "text-[var(--color-primary)] border-b-2 border-blue-600"
                : "text-[var(--color-foreground)]/60 hover:text-[var(--color-foreground)]"
            }`}
          >
            <BarChart3Icon className="w-4 h-4" />
            Estadísticas Grupales
          </button>
          <button
            onClick={() => setAttendanceView("daily")}
            className={`flex items-center gap-2 px-4 py-2 font-medium transition-colors ${
              attendanceView === "daily"
                ? "text-[var(--color-primary)] border-b-2 border-blue-600"
                : "text-[var(--color-foreground)]/60 hover:text-[var(--color-foreground)]"
            }`}
          >
            <ClipboardListIcon className="w-4 h-4" />
            Pasar Lista
          </button>
        </div>
      </div>

      {/* Attendance Content */}
      <div className="mt-6">
        {attendanceView === "stats" ? (
          <GroupAttendanceStatsView
            groupId={groupId}
            totalStudents={groupStudents.length}
          />
        ) : (
          <div>
            <AtRiskStudentsWidget groupId={groupId} threshold={threshold} />
            <DailyAttendanceForm
              groupId={groupId}
              students={groupStudents}
              onRefresh={() => {}}
            />
          </div>
        )}
      </div>
    </div>
  );
}
