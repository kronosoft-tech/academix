import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface RegisterRequest {
  name: string;
  email: string;
  password: string;
}

interface RegisterResponse {
  success: boolean;
  user?: {
    id: string;
    email: string;
    name: string;
  };
  error?: string;
}

interface UseRegisterReturn {
  register: (name: string, email: string, password: string) => Promise<{ success: boolean; error?: string }>;
  isLoading: boolean;
  error: string | null;
}

export function useRegister(): UseRegisterReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const register = async (name: string, email: string, password: string) => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await invoke<RegisterResponse>("register_user", {
        request: {
          name: name.trim(),
          email: email.trim().toLowerCase(),
          password,
        } as RegisterRequest,
      });

      if (response.success) {
        // Optionally redirect to login or auto-login
        return { success: true };
      } else {
        const errorMessage = response.error || "Error al crear la cuenta";
        setError(errorMessage);
        return { success: false, error: errorMessage };
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Error al crear la cuenta";
      setError(errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setIsLoading(false);
    }
  };

  return {
    register,
    isLoading,
    error,
  };
}