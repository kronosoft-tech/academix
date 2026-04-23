import { useState, useEffect } from "react";
import { SaveIcon, CalendarIcon } from "lucide-react";
import { Button } from "../../../shared/ui/components/Button";
import { useAttendance, AttendanceStatus } from "../hooks/useAttendance";
import type { Student } from "../../../shared/types/Student";

interface Props {
  groupId: string;
  students: Student[];
  onRefresh?: () => void;
}

type LocalAttendanceStatus = "none" | AttendanceStatus;

interface StudentAttendance {
  studentId: string;
  status: LocalAttendanceStatus;
}

export default function DailyAttendanceForm({ groupId, students, onRefresh }: Props) {
  const [selectedDate, setSelectedDate] = useState(new Date().toISOString().split("T")[0]);
  const [studentAttendance, setStudentAttendance] = useState<Map<string, StudentAttendance>>(
    new Map()
  );
  const [isSaving, setIsSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<{ type: "success" | "error"; text: string } | null>(
    null
  );
  const [hasChanges, setHasChanges] = useState(false);

  const { saveBatchAttendance, getAttendanceByGroupAndDate } = useAttendance();

  // Get students sorted by last name
  const sortedStudents = [...students].sort((a, b) => {
    const lastNameA = a.name.split(" ").pop() || a.name;
    const lastNameB = b.name.split(" ").pop() || b.name;
    return lastNameA.localeCompare(lastNameB);
  });

  // Load existing attendance when date changes
  useEffect(() => {
    const loadAttendance = async () => {
      const { records, error } = await getAttendanceByGroupAndDate(groupId, selectedDate);
      if (error) {
        console.error("Failed to load attendance:", error);
        return;
      }

      const newMap = new Map<string, StudentAttendance>();
      for (const record of records) {
        newMap.set(record.studentId, {
          studentId: record.studentId,
          status: record.status,
        });
      }
      setStudentAttendance(newMap);
      setHasChanges(false);
    };

    loadAttendance();
  }, [groupId, selectedDate, getAttendanceByGroupAndDate]);

  const handleAttendanceChange = (studentId: string, status: LocalAttendanceStatus) => {
    setStudentAttendance((prev) => {
      const newMap = new Map(prev);
      newMap.set(studentId, { studentId, status });
      return newMap;
    });
    setHasChanges(true);
    setSaveMessage(null);
  };

  const handleSave = async () => {
    setIsSaving(true);
    setSaveMessage(null);

    try {
      const records = sortedStudents
        .map((student) => {
          const attendance = studentAttendance.get(student.id);
          if (!attendance || attendance.status === "none") return null;

          return {
            studentId: student.id,
            groupId: groupId,
            date: selectedDate,
            status: attendance.status as AttendanceStatus,
            notes: undefined,
          };
        })
        .filter((r) => r !== null);

      if (records.length === 0) {
        setSaveMessage({ type: "error", text: "No hay estudiantes con asistencia registrada" });
        setIsSaving(false);
        return;
      }

      const result = await saveBatchAttendance(records);

      if (result.success) {
        setSaveMessage({ type: "success", text: "Asistencia guardada correctamente" });
        setHasChanges(false);
        onRefresh?.();
      } else {
        setSaveMessage({ type: "error", text: result.error || "Error al guardar" });
      }
    } catch (err) {
      setSaveMessage({
        type: "error",
        text: err instanceof Error ? err.message : "Error al guardar",
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="bg-[var(--color-background)] rounded-lg shadow p-6">
      <div className="flex justify-between items-center mb-4">
        <div className="flex items-center gap-3">
          <CalendarIcon className="w-5 h-5 text-[var(--color-primary)]" />
          <h3 className="text-lg font-semibold">Pasar Lista</h3>
        </div>
        <div className="flex items-center gap-4">
          <input
            type="date"
            value={selectedDate}
            onChange={(e) => setSelectedDate(e.target.value)}
            className="border rounded-lg px-3 py-2"
          />
          <Button onClick={handleSave} disabled={isSaving || !hasChanges}>
            <SaveIcon className="w-4 h-4 mr-2" />
            {isSaving ? "Guardando..." : "Guardar"}
          </Button>
        </div>
      </div>

      {saveMessage && (
        <div
          className={`mb-4 px-4 py-2 rounded-lg text-sm ${
            saveMessage.type === "success"
              ? "bg-green-50 text-green-700 border border-green-200"
              : "bg-red-50 text-red-700 border border-red-200"
          }`}
        >
          {saveMessage.text}
        </div>
      )}

      <table className="w-full">
        <thead>
          <tr className="border-b bg-[var(--color-foreground)]/5">
            <th className="text-left py-3 px-4 font-medium text-[var(--color-foreground)]/80 w-12">#</th>
            <th className="text-left py-3 px-4 font-medium text-[var(--color-foreground)]/80">Estudiante</th>
            <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Presente</th>
            <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Ausente</th>
            <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Tarde</th>
            <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Justif.</th>
          </tr>
        </thead>
        <tbody>
          {sortedStudents.length === 0 ? (
            <tr>
              <td colSpan={6} className="text-center text-[var(--color-foreground)]/60 py-8">
                No hay estudiantes inscritos en este grupo
              </td>
            </tr>
          ) : (
            sortedStudents.map((student, index) => {
              const currentStatus = studentAttendance.get(student.id)?.status || "none";
              return (
                <tr key={student.id} className="border-b hover:bg-[var(--color-foreground)]/5">
                  <td className="py-3 px-4 text-[var(--color-foreground)]/60 text-sm">{index + 1}</td>
                  <td className="py-3 px-4">{student.name}</td>
                  <td className="text-center">
                    <input
                      type="radio"
                      name={`attendance-${student.id}`}
                      checked={currentStatus === "present"}
                      onChange={() => handleAttendanceChange(student.id, "present")}
                      className="rounded text-green-600 w-5 h-5 cursor-pointer"
                    />
                  </td>
                  <td className="text-center">
                    <input
                      type="radio"
                      name={`attendance-${student.id}`}
                      checked={currentStatus === "absent"}
                      onChange={() => handleAttendanceChange(student.id, "absent")}
                      className="rounded text-red-600 w-5 h-5 cursor-pointer"
                    />
                  </td>
                  <td className="text-center">
                    <input
                      type="radio"
                      name={`attendance-${student.id}`}
                      checked={currentStatus === "late"}
                      onChange={() => handleAttendanceChange(student.id, "late")}
                      className="rounded text-yellow-600 w-5 h-5 cursor-pointer"
                    />
                  </td>
                  <td className="text-center">
                    <input
                      type="radio"
                      name={`attendance-${student.id}`}
                      checked={currentStatus === "excused"}
                      onChange={() => handleAttendanceChange(student.id, "excused")}
                      className="rounded text-[var(--color-primary)] w-5 h-5 cursor-pointer"
                    />
                  </td>
                </tr>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  );
}
