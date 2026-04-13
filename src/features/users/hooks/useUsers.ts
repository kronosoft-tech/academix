import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { User } from "../../../shared/types/User";

interface BackendUserDto {
  id: string;
  email: string;
  name: string;
  role: string;
  is_active: boolean;
}

interface UseUsersReturn {
  users: User[];
  isLoading: boolean;
  error: string | null;
  createUser: (data: CreateUserInput) => Promise<{ success: boolean; error?: string }>;
  updateUser: (id: string, data: UpdateUserInput) => Promise<{ success: boolean; error?: string }>;
  deleteUser: (id: string) => Promise<{ success: boolean; error?: string }>;
  refetch: () => void;
}

export interface CreateUserInput {
  email: string;
  password: string;
  name: string;
  role: string;
}

export interface UpdateUserInput {
  name?: string;
  email?: string;
}

function mapBackendToFrontend(dto: BackendUserDto): User {
  return {
    id: dto.id,
    email: dto.email,
    name: dto.name,
    role: dto.role as User["role"],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

export function useUsers(): UseUsersReturn {
  const [users, setUsers] = useState<User[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchUsers = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendUserDto[] | null;
        error: string | null;
      }>("list_users");

      if (response.success && response.data) {
        setUsers(response.data.map(mapBackendToFrontend));
      } else {
        setError(response.error || "Failed to fetch users");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch users");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchUsers();
  }, [fetchUsers]);

  const createUser = async (data: CreateUserInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendUserDto | null;
        error: string | null;
      }>("create_user", {
        request: {
          email: data.email,
          password: data.password,
          name: data.name,
          role: data.role,
        },
      });

      if (response.success) {
        await fetchUsers();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to create user" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to create user" };
    } finally {
      setIsLoading(false);
    }
  };

  const updateUser = async (id: string, data: UpdateUserInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendUserDto | null;
        error: string | null;
      }>("update_user", {
        id,
        request: {
          name: data.name ?? null,
          email: data.email ?? null,
        },
      });

      if (response.success) {
        await fetchUsers();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to update user" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to update user" };
    } finally {
      setIsLoading(false);
    }
  };

  const deleteUser = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("delete_user", { id });

      if (response.success) {
        await fetchUsers();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to delete user" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to delete user" };
    } finally {
      setIsLoading(false);
    }
  };

  return {
    users,
    isLoading,
    error,
    createUser,
    updateUser,
    deleteUser,
    refetch: fetchUsers,
  };
}
