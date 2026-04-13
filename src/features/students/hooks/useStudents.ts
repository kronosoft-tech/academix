import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Student, CreateStudentInput, UpdateStudentInput } from "../../../shared/types/Student";

interface StudentFilters {
  search?: string;
  documentType?: string;
}

interface BackendStudentDto {
  id: string;
  user_id: string;
  first_name: string;
  last_name: string;
  document_type: string;
  document_number: string;
  email: string;
  phone: string | null;
  address: string | null;
  birth_date: string | null;
  guardian_name: string | null;
  guardian_document: string | null;
  guardian_phone: string | null;
  course_id: string | null;
  group_id: string | null;
  course_name: string | null;
  group_name: string | null;
  active: boolean;
  created_at: string;
  updated_at: string;
}

interface UseStudentsReturn {
  students: Student[];
  isLoading: boolean;
  error: string | null;
  filters: StudentFilters;
  setFilters: (filters: StudentFilters) => void;
  createStudent: (data: CreateStudentInput) => Promise<{ success: boolean; error?: string }>;
  updateStudent: (id: string, data: UpdateStudentInput) => Promise<{ success: boolean; error?: string }>;
  deleteStudent: (id: string) => Promise<{ success: boolean; error?: string }>;
  refetch: () => void;
}

function mapBackendToFrontend(dto: BackendStudentDto): Student {
  return {
    id: dto.id,
    userId: dto.user_id,
    name: `${dto.first_name} ${dto.last_name}`.trim(),
    firstName: dto.first_name,
    lastName: dto.last_name,
    documentNumber: dto.document_number,
    documentType: dto.document_type as Student["documentType"],
    email: dto.email || "",
    phone: dto.phone ?? undefined,
    address: dto.address ?? undefined,
    birthDate: dto.birth_date ?? undefined,
    guardianName: dto.guardian_name ?? undefined,
    guardianDocument: dto.guardian_document ?? undefined,
    guardianPhone: dto.guardian_phone ?? undefined,
    courseId: dto.course_id ?? undefined,
    groupId: dto.group_id ?? undefined,
    courseName: dto.course_name ?? undefined,
    groupName: dto.group_name ?? undefined,
    createdAt: dto.created_at,
    updatedAt: dto.updated_at,
  };
}

export function useStudents(): UseStudentsReturn {
  const [students, setStudents] = useState<Student[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filters, setFilters] = useState<StudentFilters>({});

  const fetchStudents = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendStudentDto[] | null;
        error: string | null;
      }>("list_students");

      if (response.success && response.data) {
        setStudents(response.data.map(mapBackendToFrontend));
      } else {
        setError(response.error || "Failed to fetch students");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch students");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStudents();
  }, [fetchStudents]);

  const createStudent = async (data: CreateStudentInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const nameParts = data.name.split(" ");
      const firstName = nameParts[0] || "";
      const lastName = nameParts.slice(1).join(" ") || "";

      const response = await invoke<{
        success: boolean;
        data: BackendStudentDto | null;
        error: string | null;
      }>("create_student", {
        request: {
          user_id: crypto.randomUUID(),
          first_name: firstName,
          last_name: lastName,
          document_type: data.documentType,
          document_number: data.documentNumber,
          email: data.email,
          phone: data.phone ?? null,
          address: data.address ?? null,
          birth_date: data.birthDate ?? null,
          guardian_name: data.guardianName ?? null,
          guardian_document: data.guardianDocument ?? null,
          guardian_phone: data.guardianPhone ?? null,
          course_id: data.courseId ?? null,
          group_id: data.groupId ?? null,
        },
      });

      if (response.success) {
        await fetchStudents();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to create student" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to create student" };
    } finally {
      setIsLoading(false);
    }
  };

  const updateStudent = async (id: string, data: UpdateStudentInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const nameParts = (data.name || "").split(" ");
      const firstName = nameParts[0];
      const lastName = nameParts.slice(1).join(" ");

      const response = await invoke<{
        success: boolean;
        data: BackendStudentDto | null;
        error: string | null;
      }>("update_student", {
        id,
        request: {
          first_name: firstName ?? null,
          last_name: lastName ?? null,
          email: data.email ?? null,
          phone: data.phone ?? null,
          address: data.address ?? null,
          birth_date: data.birthDate ?? null,
          guardian_name: data.guardianName ?? null,
          guardian_document: data.guardianDocument ?? null,
          guardian_phone: data.guardianPhone ?? null,
          course_id: data.courseId ?? null,
          group_id: data.groupId ?? null,
        },
      });

      if (response.success) {
        await fetchStudents();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to update student" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to update student" };
    } finally {
      setIsLoading(false);
    }
  };

  const deleteStudent = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("delete_student", { id });

      if (response.success) {
        await fetchStudents();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to delete student" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to delete student" };
    } finally {
      setIsLoading(false);
    }
  };

  return {
    students,
    isLoading,
    error,
    filters,
    setFilters,
    createStudent,
    updateStudent,
    deleteStudent,
    refetch: fetchStudents,
  };
}
