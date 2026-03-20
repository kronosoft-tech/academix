import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Course, CreateCourseInput, UpdateCourseInput } from "../../../shared/types/Course";

interface BackendCourseDto {
  id: string;
  name: string;
  code: string;
  credits: number;
  price: number;
  duration: number;
  description: string | null;
  status: string;
}

interface UseCoursesReturn {
  courses: Course[];
  archivedCourses: Course[];
  isLoading: boolean;
  error: string | null;
  createCourse: (data: CreateCourseInput) => Promise<{ success: boolean; error?: string }>;
  updateCourse: (id: string, data: UpdateCourseInput) => Promise<{ success: boolean; error?: string }>;
  archiveCourse: (id: string) => Promise<{ success: boolean; error?: string }>;
  restoreCourse: (id: string) => Promise<{ success: boolean; error?: string }>;
  hardDeleteCourse: (id: string) => Promise<{ success: boolean; error?: string }>;
  refetch: () => void;
  refetchArchived: () => void;
}

function mapBackendToFrontend(dto: BackendCourseDto): Course {
  return {
    id: dto.id,
    name: dto.name,
    code: dto.code,
    description: dto.description ?? undefined,
    price: dto.price || 200000,
    duration: dto.duration || 0,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

export function useCourses(): UseCoursesReturn {
  const [courses, setCourses] = useState<Course[]>([]);
  const [archivedCourses, setArchivedCourses] = useState<Course[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchCourses = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendCourseDto[] | null;
        error: string | null;
      }>("list_courses");

      if (response.success && response.data) {
        setCourses(response.data.map(mapBackendToFrontend));
      } else {
        setError(response.error || "Failed to fetch courses");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch courses");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchCourses();
    fetchArchivedCourses();
  }, [fetchCourses]);

  const fetchArchivedCourses = useCallback(async () => {
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendCourseDto[] | null;
        error: string | null;
      }>("list_archived_courses");

      if (response.success && response.data) {
        setArchivedCourses(response.data.map(mapBackendToFrontend));
      }
    } catch (err) {
      console.error("Failed to fetch archived courses:", err);
    }
  }, []);

  const createCourse = async (data: CreateCourseInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendCourseDto | null;
        error: string | null;
      }>("create_course", {
        request: {
          name: data.name,
          code: data.code,
          credits: 1, // Default credits
          description: data.description ?? null,
          price: data.price,
          duration: data.duration,
        },
      });

      if (response.success) {
        await fetchCourses();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to create course" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to create course" };
    } finally {
      setIsLoading(false);
    }
  };

  const updateCourse = async (id: string, data: UpdateCourseInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendCourseDto | null;
        error: string | null;
      }>("update_course", {
        id,
        request: {
          name: data.name ?? null,
          description: data.description ?? null,
          price: data.price ?? null,
          duration: data.duration ?? null,
        },
      });

      if (response.success) {
        await fetchCourses();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to update course" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to update course" };
    } finally {
      setIsLoading(false);
    }
  };

  const archiveCourse = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("archive_course", { id });

      if (response.success) {
        await fetchCourses();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to archive course" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to archive course" };
    } finally {
      setIsLoading(false);
    }
  };

  const restoreCourse = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("restore_course", { id });

      if (response.success) {
        await fetchCourses();
        await fetchArchivedCourses();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to restore course" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to restore course" };
    } finally {
      setIsLoading(false);
    }
  };

  const hardDeleteCourse = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("hard_delete_course", { id });

      if (response.success) {
        await fetchArchivedCourses();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to delete course" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to delete course" };
    } finally {
      setIsLoading(false);
    }
  };

  return {
    courses,
    archivedCourses,
    isLoading,
    error,
    createCourse,
    updateCourse,
    archiveCourse,
    restoreCourse,
    hardDeleteCourse,
    refetch: fetchCourses,
    refetchArchived: fetchArchivedCourses,
  };
}
