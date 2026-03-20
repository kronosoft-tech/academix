import { useState } from "react";
import { ArrowLeftIcon, PencilIcon, BarChart3Icon, ClipboardListIcon } from "lucide-react";
import type { Group } from "../../../shared/types/Group";
import type { Course } from "../../../shared/types/Course";
import type { User } from "../../../shared/types/User";
import type { Student } from "../../../shared/types/Student";
import { Button } from "../../../shared/ui/components/Button";
import GroupAttendanceStatsView from "./GroupAttendanceStatsView";
import DailyAttendanceForm from "./DailyAttendanceForm";

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
          <button onClick={onBack} className="p-2 hover:bg-gray-100 rounded-lg">
            <ArrowLeftIcon className="w-5 h-5" />
          </button>
          <div>
            <h1 className="text-2xl font-bold">{group.name}</h1>
            <p className="text-gray-500">
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
        <div className="bg-white p-4 rounded-lg shadow">
          <p className="text-sm text-gray-500">Curso</p>
          <p className="font-semibold">{course?.name || "-"}</p>
        </div>
        <div className="bg-white p-4 rounded-lg shadow">
          <p className="text-sm text-gray-500">Profesor</p>
          <p className="font-semibold">{professorName}</p>
        </div>
        <div className="bg-white p-4 rounded-lg shadow">
          <p className="text-sm text-gray-500">Cupo</p>
          <p className="font-semibold">
            {group.currentStudents || 0} / {group.maxStudents}
          </p>
        </div>
        <div className="bg-white p-4 rounded-lg shadow">
          <p className="text-sm text-gray-500">Estado</p>
          <p className="font-semibold capitalize">
            {group.status === "open"
              ? "Activo"
              : group.status === "completed"
                ? "Completado"
                : "Cerrado"}
          </p>
        </div>
      </div>

      {/* Attendance Section Tabs */}
      <div className="mb-6">
        <div className="flex gap-2 border-b">
          <button
            onClick={() => setAttendanceView("stats")}
            className={`flex items-center gap-2 px-4 py-2 font-medium transition-colors ${
              attendanceView === "stats"
                ? "text-blue-600 border-b-2 border-blue-600"
                : "text-gray-500 hover:text-gray-700"
            }`}
          >
            <BarChart3Icon className="w-4 h-4" />
            Estadísticas Grupales
          </button>
          <button
            onClick={() => setAttendanceView("daily")}
            className={`flex items-center gap-2 px-4 py-2 font-medium transition-colors ${
              attendanceView === "daily"
                ? "text-blue-600 border-b-2 border-blue-600"
                : "text-gray-500 hover:text-gray-700"
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
          <DailyAttendanceForm
            groupId={groupId}
            students={groupStudents}
            onRefresh={() => {}}
          />
        )}
      </div>
    </div>
  );
}
