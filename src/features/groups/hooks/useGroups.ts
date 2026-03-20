import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Group, CreateGroupInput, UpdateGroupInput } from "../../../shared/types/Group";

interface BackendGroupDto {
  id: string;
  course_id: string;
  name: string;
  professor_id: string;
  schedule: string | null;
  days: string[] | null;
  start_time: string | null;
  end_time: string | null;
  max_students: number;
  current_students: number;
  status: string;
}

interface UseGroupsReturn {
  groups: Group[];
  isLoading: boolean;
  error: string | null;
  createGroup: (data: CreateGroupInput) => Promise<{ success: boolean; error?: string }>;
  updateGroup: (id: string, data: UpdateGroupInput) => Promise<{ success: boolean; error?: string }>;
  deleteGroup: (id: string) => Promise<{ success: boolean; error?: string }>;
  refetch: () => void;
}

function mapBackendToFrontend(dto: BackendGroupDto): Group {
  return {
    id: dto.id,
    courseId: dto.course_id,
    name: dto.name,
    professorId: dto.professor_id || undefined,
    schedule: dto.schedule ?? "",
    days: dto.days ?? undefined,
    startTime: dto.start_time ?? undefined,
    endTime: dto.end_time ?? undefined,
    maxStudents: dto.max_students,
    currentStudents: dto.current_students,
    status: dto.status as Group["status"],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

export function useGroups(): UseGroupsReturn {
  const [groups, setGroups] = useState<Group[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchGroups = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendGroupDto[] | null;
        error: string | null;
      }>("list_groups");

      if (response.success && response.data) {
        setGroups(response.data.map(mapBackendToFrontend));
      } else {
        setError(response.error || "Failed to fetch groups");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch groups");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchGroups();
  }, [fetchGroups]);

  const createGroup = async (data: CreateGroupInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendGroupDto | null;
        error: string | null;
      }>("create_group", {
        request: {
          course_id: data.courseId,
          name: data.name,
          professor_id: data.professorId || null,
          schedule: data.schedule ?? null,
          days: data.days ?? null,
          start_time: data.startTime ?? null,
          end_time: data.endTime ?? null,
          max_students: data.maxStudents ?? 20,
        },
      });

      if (response.success) {
        await fetchGroups();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to create group" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to create group" };
    } finally {
      setIsLoading(false);
    }
  };

  const updateGroup = async (id: string, data: UpdateGroupInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendGroupDto | null;
        error: string | null;
      }>("update_group", {
        id,
        request: {
          name: data.name ?? null,
          schedule: data.schedule ?? null,
          days: data.days ?? null,
          start_time: data.startTime ?? null,
          end_time: data.endTime ?? null,
          max_students: data.maxStudents ?? null,
          status: data.status ?? null,
        },
      });

      if (response.success) {
        await fetchGroups();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to update group" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to update group" };
    } finally {
      setIsLoading(false);
    }
  };

  const deleteGroup = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("delete_group", { id });

      if (response.success) {
        await fetchGroups();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to delete group" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to delete group" };
    } finally {
      setIsLoading(false);
    }
  };

  return {
    groups,
    isLoading,
    error,
    createGroup,
    updateGroup,
    deleteGroup,
    refetch: fetchGroups,
  };
}
