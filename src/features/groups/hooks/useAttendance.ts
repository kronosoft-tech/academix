import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export type AttendanceStatus = "present" | "absent" | "late" | "excused";

export interface AttendanceRecord {
  id: string;
  studentId: string;
  groupId: string;
  date: string;
  status: AttendanceStatus;
  notes?: string;
}

export interface SaveAttendanceInput {
  studentId: string;
  groupId: string;
  date: string;
  status: AttendanceStatus;
  notes?: string;
}

export interface GroupAttendanceStats {
  groupId: string;
  totalRecords: number;
  presentCount: number;
  presentPercentage: number;
  absentCount: number;
  absentPercentage: number;
  lateCount: number;
  latePercentage: number;
  excusedCount: number;
  excusedPercentage: number;
  totalStudents: number;
  totalSessions: number;
}

interface BackendAttendanceDto {
  id: string;
  student_id: string;
  group_id: string;
  date: string;
  status: string;
  notes: string | null;
}

interface BackendGroupStatsDto {
  group_id: string;
  total_records: number;
  present_count: number;
  present_percentage: number;
  absent_count: number;
  absent_percentage: number;
  late_count: number;
  late_percentage: number;
  excused_count: number;
  excused_percentage: number;
  total_students: number;
  total_sessions: number;
}

interface UseAttendanceReturn {
  saveAttendance: (data: SaveAttendanceInput) => Promise<{ success: boolean; error?: string }>;
  saveBatchAttendance: (
    records: SaveAttendanceInput[]
  ) => Promise<{ success: boolean; error?: string }>;
  getAttendanceByGroupAndDate: (
    groupId: string,
    date: string
  ) => Promise<{ records: AttendanceRecord[]; error?: string | null }>;
  getGroupStats: (
    groupId: string,
    totalStudents: number
  ) => Promise<{ stats: GroupAttendanceStats | null; error?: string | null }>;
}

function mapBackendToFrontend(dto: BackendAttendanceDto): AttendanceRecord {
  return {
    id: dto.id,
    studentId: dto.student_id,
    groupId: dto.group_id,
    date: dto.date,
    status: dto.status as AttendanceStatus,
    notes: dto.notes ?? undefined,
  };
}

export function useAttendance(): UseAttendanceReturn {
  const [_isLoading, setIsLoading] = useState(false);

  const saveAttendance = useCallback(
    async (data: SaveAttendanceInput): Promise<{ success: boolean; error?: string }> => {
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
          return { success: true };
        } else {
          return { success: false, error: response.error || "Failed to save attendance" };
        }
      } catch (err) {
        return {
          success: false,
          error: err instanceof Error ? err.message : "Failed to save attendance",
        };
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const saveBatchAttendance = useCallback(
    async (
      records: SaveAttendanceInput[]
    ): Promise<{ success: boolean; error?: string }> => {
      setIsLoading(true);
      try {
        // Save each record individually
        for (const record of records) {
          const response = await invoke<{
            success: boolean;
            data: BackendAttendanceDto | null;
            error: string | null;
          }>("create_attendance", {
            request: {
              student_id: record.studentId,
              group_id: record.groupId,
              date: record.date,
              status: record.status,
              notes: record.notes ?? null,
            },
          });

          if (!response.success) {
            return { success: false, error: response.error || "Failed to save attendance" };
          }
        }

        return { success: true };
      } catch (err) {
        return {
          success: false,
          error: err instanceof Error ? err.message : "Failed to save batch attendance",
        };
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  const getAttendanceByGroupAndDate = useCallback(
    async (
      groupId: string,
      date: string
    ): Promise<{ records: AttendanceRecord[]; error: string | null }> => {
      try {
        const response = await invoke<{
          success: boolean;
          data: BackendAttendanceDto[] | null;
          error: string | null;
        }>("list_attendance_by_group_date", {
          groupId,
          date,
        });

        if (response.success && response.data) {
          return {
            records: response.data.map(mapBackendToFrontend),
            error: null,
          };
        } else {
          return { records: [], error: response.error || "Failed to fetch attendance" };
        }
      } catch (err) {
        return {
          records: [],
          error: err instanceof Error ? err.message : "Failed to fetch attendance",
        };
      }
    },
    []
  );

  const getGroupStats = useCallback(
    async (
      groupId: string,
      totalStudents: number
    ): Promise<{ stats: GroupAttendanceStats | null; error: string | null }> => {
      try {
        const response = await invoke<{
          success: boolean;
          data: BackendGroupStatsDto | null;
          error: string | null;
        }>("get_group_attendance_stats", {
          groupId,
          totalStudents,
        });

        if (response.success && response.data) {
          return {
            stats: {
              groupId: response.data.group_id,
              totalRecords: response.data.total_records,
              presentCount: response.data.present_count,
              presentPercentage: response.data.present_percentage,
              absentCount: response.data.absent_count,
              absentPercentage: response.data.absent_percentage,
              lateCount: response.data.late_count,
              latePercentage: response.data.late_percentage,
              excusedCount: response.data.excused_count,
              excusedPercentage: response.data.excused_percentage,
              totalStudents: response.data.total_students,
              totalSessions: response.data.total_sessions,
            },
            error: null,
          };
        } else {
          return { stats: null, error: response.error || "Failed to fetch stats" };
        }
      } catch (err) {
        return {
          stats: null,
          error: err instanceof Error ? err.message : "Failed to fetch stats",
        };
      }
    },
    []
  );

  return {
    saveAttendance,
    saveBatchAttendance,
    getAttendanceByGroupAndDate,
    getGroupStats,
  };
}
