import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Attendance, CreateAttendanceInput, UpdateAttendanceInput } from "../../../shared/types/Attendance";

interface BackendAttendanceDto {
  id: string;
  student_id: string;
  group_id: string;
  date: string;
  status: string;
  notes: string | null;
}

interface UseAttendanceReturn {
  attendance: Attendance[];
  isLoading: boolean;
  error: string | null;
  createAttendance: (data: CreateAttendanceInput) => Promise<{ success: boolean; error?: string }>;
  updateAttendance: (id: string, data: UpdateAttendanceInput) => Promise<{ success: boolean; error?: string }>;
  deleteAttendance: (id: string) => Promise<{ success: boolean; error?: string }>;
  refetch: () => void;
}

function mapBackendToFrontend(dto: BackendAttendanceDto): Attendance {
  return {
    id: dto.id,
    studentId: dto.student_id,
    groupId: dto.group_id,
    date: dto.date,
    status: dto.status as Attendance["status"],
    notes: dto.notes ?? undefined,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

export function useAttendance(): UseAttendanceReturn {
  const [attendance, setAttendance] = useState<Attendance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchAttendance = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendAttendanceDto[] | null;
        error: string | null;
      }>("list_attendances");

      if (response.success && response.data) {
        setAttendance(response.data.map(mapBackendToFrontend));
      } else {
        setError(response.error || "Failed to fetch attendance");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch attendance");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAttendance();
  }, [fetchAttendance]);

  const createAttendance = async (data: CreateAttendanceInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendAttendanceDto | null;
        error: string | null;
      }>("create_attendance", {
        request: {
          student_id: data.studentId,
          group_id: data.groupId,
          date: data.date,
          status: data.status,
          notes: data.notes ?? null,
        },
      });

      if (response.success) {
        await fetchAttendance();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to create attendance" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to create attendance" };
    } finally {
      setIsLoading(false);
    }
  };

  const updateAttendance = async (id: string, data: UpdateAttendanceInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendAttendanceDto | null;
        error: string | null;
      }>("update_attendance", {
        id,
        request: {
          status: data.status ?? null,
          notes: data.notes ?? null,
        },
      });

      if (response.success) {
        await fetchAttendance();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to update attendance" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to update attendance" };
    } finally {
      setIsLoading(false);
    }
  };

  const deleteAttendance = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("delete_attendance", { id });

      if (response.success) {
        await fetchAttendance();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to delete attendance" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to delete attendance" };
    } finally {
      setIsLoading(false);
    }
  };

  return {
    attendance,
    isLoading,
    error,
    createAttendance,
    updateAttendance,
    deleteAttendance,
    refetch: fetchAttendance,
  };
}
