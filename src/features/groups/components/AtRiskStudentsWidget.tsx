import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Badge } from "../../../shared/ui/components/Badge";
import { Spinner } from "../../../shared/ui/components/Spinner";

interface Student {
  id: string;
  name: string;
}

interface AbsenceCount {
  student_id: string;
  absence_count: number;
}

interface AtRiskStudentsResponse {
  success: boolean;
  data: AbsenceCount[] | null;
  error: string | null;
}

interface StudentsResponse {
  success: boolean;
  data: { id: string; first_name: string; last_name: string }[] | null;
  error: string | null;
}

interface AtRiskStudentsWidgetProps {
  groupId: string;
  threshold: number;
}

export default function AtRiskStudentsWidget({ groupId, threshold }: AtRiskStudentsWidgetProps) {
  const [atRiskStudents, setAtRiskStudents] = useState<(AbsenceCount & { student_name: string })[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchAtRisk = async () => {
      setIsLoading(true);
      try {
        const [absencesRes, studentsRes] = await Promise.all([
          invoke<AtRiskStudentsResponse>("count_group_absences", { groupId }),
          invoke<StudentsResponse>("list_students"),
        ]);

        if (absencesRes.success && absencesRes.data && studentsRes.success && studentsRes.data) {
          const studentMap = new Map<string, Student>(
            studentsRes.data.map((s) => [s.id, { id: s.id, name: `${s.first_name} ${s.last_name}`.trim() }])
          );

          const atRisk = absencesRes.data
            .filter((s) => s.absence_count > threshold)
            .sort((a, b) => b.absence_count - a.absence_count)
            .map((s) => ({
              ...s,
              student_name: studentMap.get(s.student_id)?.name ?? s.student_id,
            }));
          setAtRiskStudents(atRisk);
        }
      } catch {
        setAtRiskStudents([]);
      } finally {
        setIsLoading(false);
      }
    };
    fetchAtRisk();
  }, [groupId, threshold]);

  return (
    <div className="bg-[var(--color-background)] rounded-lg shadow border p-4 mb-4">
      <h3 className="text-sm font-semibold text-[var(--color-foreground)] mb-3">
        Estudiantes en riesgo ({atRiskStudents.length})
      </h3>
      {isLoading ? (
        <div className="flex justify-center py-4">
          <Spinner size="sm" />
        </div>
      ) : atRiskStudents.length === 0 ? (
        <p className="text-sm text-[var(--color-foreground)]/60 py-4 text-center">
          No hay estudiantes en riesgo
        </p>
      ) : (
        <div className="space-y-2">
          {atRiskStudents.map((student) => (
            <div
              key={student.student_id}
              className="flex items-center justify-between py-2 px-3 rounded-lg bg-[var(--color-foreground)]/5"
            >
              <span className="text-sm text-[var(--color-foreground)]">
                {student.student_name}
              </span>
              <Badge variant="danger">
                Más de {threshold} faltas
              </Badge>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
