import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { StudentAbsenceCount } from "../../../shared/types/AttendanceWarning";

interface AbsenceCountResponse {
  success: boolean;
  data: number | null;
  error: string | null;
}

interface GroupAbsenceCountsResponse {
  success: boolean;
  data: { student_id: string; absence_count: number }[] | null;
  error: string | null;
}

interface UseStudentAbsencesReturn {
  getAbsenceCount: (studentId: string, groupId: string) => Promise<number>;
  getGroupAbsenceCounts: (groupId: string) => Promise<StudentAbsenceCount[]>;
  isLoading: boolean;
  error: string | null;
}

export function useStudentAbsences(): UseStudentAbsencesReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getAbsenceCount = useCallback(
    async (studentId: string, groupId: string): Promise<number> => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await invoke<AbsenceCountResponse>(
          "count_student_absences",
          { studentId, groupId }
        );
        if (response.success && response.data !== null) {
          return response.data;
        }
        setError(response.error || "Error al obtener faltas");
        return 0;
      } catch (err) {
        setError(err instanceof Error ? err.message : "Error al obtener faltas");
        return 0;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const getGroupAbsenceCounts = useCallback(
    async (groupId: string): Promise<StudentAbsenceCount[]> => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await invoke<GroupAbsenceCountsResponse>(
          "count_group_absences",
          { groupId }
        );
        if (response.success && response.data) {
          return response.data.map((item) => ({
            studentId: item.student_id,
            absenceCount: item.absence_count,
          }));
        }
        setError(response.error || "Error al obtener faltas del grupo");
        return [];
      } catch (err) {
        setError(err instanceof Error ? err.message : "Error al obtener faltas del grupo");
        return [];
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  return { getAbsenceCount, getGroupAbsenceCounts, isLoading, error };
}
