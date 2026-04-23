import { useState, useEffect } from "react";
import { ArrowLeftIcon } from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { useGroups } from "../hooks/useGroups";
import { useCourses } from "../../courses/hooks/useCourses";
import { useStudents } from "../../students/hooks/useStudents";
import { useUsers } from "../../users/hooks/useUsers";
import { Button } from "../../../shared/ui/components/Button";

interface AttendanceRecord {
  studentId: string;
  present: boolean;
  absent: boolean;
  late: boolean;
  justified: boolean;
  date: string;
}

export default function GroupDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { groups } = useGroups();
  const { courses } = useCourses();
  const { students } = useStudents();
  const { users } = useUsers();
  
  const group = groups.find(g => g.id === id);
  const course = courses.find(c => c.id === group?.courseId);
  const groupStudents = students.filter(s => s.groupId === id);
  const professor = users.find(u => u.id === group?.professorId);
  
  const [attendance, setAttendance] = useState<Record<string, AttendanceRecord>>({});
  const [selectedDate, setSelectedDate] = useState(new Date().toISOString().split('T')[0]);
  
  useEffect(() => {
    const newAttendance: Record<string, AttendanceRecord> = {};
    groupStudents.forEach(student => {
      const key = `${student.id}-${selectedDate}`;
      if (!attendance[key]) {
        newAttendance[key] = {
          studentId: student.id,
          present: false,
          absent: false,
          late: false,
          justified: false,
          date: selectedDate,
        };
      }
    });
    if (Object.keys(newAttendance).length > 0) {
      setAttendance(prev => ({ ...prev, ...newAttendance }));
    }
  }, [groupStudents, selectedDate]);
  
  const toggleAttendance = (studentId: string, field: 'present' | 'absent' | 'late' | 'justified') => {
    const key = `${studentId}-${selectedDate}`;
    setAttendance(prev => ({
      ...prev,
      [key]: {
        ...prev[key],
        [field]: !prev[key]?.[field],
      }
    }));
  };
  
  const saveAttendance = async () => {
    alert("Asistencia guardada (funcionalidad pendiente)");
  };
  
  const handleEdit = () => {
    navigate(`/groups/${id}/edit`);
  };
  
  if (!group) return <div className="p-6">Grupo no encontrado</div>;
  
  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <button onClick={() => navigate(-1)} className="p-2 hover:bg-[var(--color-foreground)]/10 rounded-lg">
            <ArrowLeftIcon className="w-5 h-5" />
          </button>
          <div>
            <h1 className="text-2xl font-bold">{group.name}</h1>
            <p className="text-[var(--color-foreground)]/60">{group.schedule || "Horario no definido"}</p>
          </div>
        </div>
        <Button onClick={handleEdit}>Editar Grupo</Button>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow border">
          <p className="text-sm text-[var(--color-foreground)]/60">Curso</p>
          <p className="font-semibold">{course?.name || "-"}</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow border">
          <p className="text-sm text-[var(--color-foreground)]/60">Profesor</p>
          <p className="font-semibold">{professor?.name || "No asignado"}</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow border">
          <p className="text-sm text-[var(--color-foreground)]/60">Cupo</p>
          <p className="font-semibold">{group.currentStudents || 0} / {group.maxStudents}</p>
        </div>
        <div className="bg-[var(--color-background)] p-4 rounded-lg shadow border">
          <p className="text-sm text-[var(--color-foreground)]/60">Estado</p>
          <p className="font-semibold capitalize">{group.status === "open" ? "Activo" : group.status === "completed" ? "Completado" : "Cerrado"}</p>
        </div>
      </div>
      
      <div className="bg-[var(--color-background)] rounded-lg shadow border p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold">Control de Asistencia</h2>
          <div className="flex items-center gap-4">
            <input
              type="date"
              value={selectedDate}
              onChange={(e) => setSelectedDate(e.target.value)}
              className="border rounded-lg px-3 py-2"
            />
            <Button onClick={saveAttendance}>Guardar Asistencia</Button>
          </div>
        </div>
        
        <table className="w-full">
          <thead>
            <tr className="border-b">
              <th className="text-left py-3 px-4 font-medium text-[var(--color-foreground)]/80">Estudiante</th>
              <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Presente</th>
              <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Ausente</th>
              <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Tarde</th>
              <th className="text-center py-3 px-2 font-medium text-[var(--color-foreground)]/80 w-20">Justif.</th>
            </tr>
          </thead>
          <tbody>
            {groupStudents.map(student => {
              const key = `${student.id}-${selectedDate}`;
              const record = attendance[key] || { present: false, absent: false, late: false, justified: false };
              return (
                <tr key={student.id} className="border-b hover:bg-[var(--color-foreground)]/5">
                  <td className="py-3 px-4">{student.name}</td>
                  <td className="text-center">
                    <input
                      type="checkbox"
                      checked={record.present}
                      onChange={() => toggleAttendance(student.id, 'present')}
                      className="rounded text-green-600 w-5 h-5"
                    />
                  </td>
                  <td className="text-center">
                    <input
                      type="checkbox"
                      checked={record.absent}
                      onChange={() => toggleAttendance(student.id, 'absent')}
                      className="rounded text-red-600 w-5 h-5"
                    />
                  </td>
                  <td className="text-center">
                    <input
                      type="checkbox"
                      checked={record.late}
                      onChange={() => toggleAttendance(student.id, 'late')}
                      className="rounded text-yellow-600 w-5 h-5"
                    />
                  </td>
                  <td className="text-center">
                    <input
                      type="checkbox"
                      checked={record.justified}
                      onChange={() => toggleAttendance(student.id, 'justified')}
                      className="rounded text-[var(--color-primary)] w-5 h-5"
                    />
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
        
        {groupStudents.length === 0 && (
          <p className="text-center text-[var(--color-foreground)]/60 py-8">No hay estudiantes en este grupo.</p>
        )}
      </div>
    </div>
  );
}