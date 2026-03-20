import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect, useCallback } from "react";

type UserRole = "admin" | "gerente" | "empleado" | "profesor";

export interface User {
  id: string;
  email: string;
  name: string;
  role: UserRole;
}

interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
}

const AUTH_STORAGE_KEY = "academix_auth";

function getStoredAuth(): AuthState {
  try {
    const stored = localStorage.getItem(AUTH_STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      // Validate we have required fields
      if (parsed.user && parsed.token && parsed.isAuthenticated) {
        return { ...parsed, isLoading: false };
      }
    }
  } catch {
    // Ignore errors
  }
  // No valid session - don't show loading forever
  return { user: null, isAuthenticated: false, isLoading: false, token: null };
}

function setStoredAuth(state: AuthState): void {
  try {
    if (state.user) {
      localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(state));
    } else {
      localStorage.removeItem(AUTH_STORAGE_KEY);
    }
  } catch {
    // Ignore errors
  }
}

let authState: AuthState = getStoredAuth();
const listeners: Set<(state: AuthState) => void> = new Set();

function notifyListeners(): void {
  listeners.forEach((listener) => listener(authState));
}

export function useAuth() {
  const [state, setState] = useState<AuthState>(authState);

  useEffect(() => {
    const listener = (newState: AuthState) => setState(newState);
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    setState((prev) => ({ ...prev, isLoading: true }));
    try {
      const response = await invoke<{
        success: boolean;
        token: string | null;
        user: User | null;
        error: string | null;
      }>("login", {
        request: { email, password },
      });

      if (response.success && response.token && response.user) {
        authState = {
          user: response.user,
          token: response.token,
          isAuthenticated: true,
          isLoading: false,
        };
        setStoredAuth(authState);
        notifyListeners();
        return { success: true };
      } else {
        authState = { user: null, token: null, isAuthenticated: false, isLoading: false };
        setStoredAuth(authState);
        notifyListeners();
        return { success: false, error: response.error || "Login failed" };
      }
    } catch (error) {
      authState = { user: null, token: null, isAuthenticated: false, isLoading: false };
      setStoredAuth(authState);
      notifyListeners();
      return { success: false, error: error instanceof Error ? error.message : "Login failed" };
    }
  }, []);

  const logout = useCallback(async () => {
    const currentToken = authState.token;
    if (currentToken) {
      try {
        await invoke("logout", { request: { token: currentToken } });
      } catch {
        // Ignore logout errors
      }
    }
    authState = { user: null, token: null, isAuthenticated: false, isLoading: false };
    setStoredAuth(authState);
    notifyListeners();
  }, []);

  return {
    user: state.user,
    isAuthenticated: state.isAuthenticated,
    isLoading: state.isLoading,
    token: state.token,
    login,
    logout,
  };
}
