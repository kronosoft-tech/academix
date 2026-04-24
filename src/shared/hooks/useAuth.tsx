import { invoke } from "@tauri-apps/api/core";
import { createContext, useContext, useState, useCallback, type ReactNode } from "react";

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

interface AuthContextValue extends AuthState {
  login: (email: string, password: string) => Promise<{ success: boolean; error?: string }>;
  logout: () => Promise<void>;
}

const AUTH_STORAGE_KEY = "academix_auth";

function getStoredAuth(): AuthState {
  try {
    const stored = localStorage.getItem(AUTH_STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (parsed.user && parsed.token && parsed.isAuthenticated) {
        return { ...parsed, isLoading: false };
      }
    }
  } catch {
    // Ignore errors
  }
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

// Create context with undefined default to force usage check
const AuthContext = createContext<AuthContextValue | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [state, setState] = useState<AuthState>(getStoredAuth);

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
        const newState = {
          user: response.user,
          token: response.token,
          isAuthenticated: true,
          isLoading: false,
        };
        setStoredAuth(newState);
        setState(newState);
        return { success: true };
      } else {
        const newState = { user: null, token: null, isAuthenticated: false, isLoading: false };
        setStoredAuth(newState);
        setState(newState);
        return { success: false, error: response.error || "Login failed" };
      }
    } catch (error) {
      const newState = { user: null, token: null, isAuthenticated: false, isLoading: false };
      setStoredAuth(newState);
      setState(newState);
      return { success: false, error: error instanceof Error ? error.message : "Login failed" };
    }
  }, []);

  const logout = useCallback(async () => {
    const currentToken = state.token;
    if (currentToken) {
      try {
        await invoke("logout", { request: { token: currentToken } });
      } catch {
        // Ignore logout errors
      }
    }
    const newState = { user: null, token: null, isAuthenticated: false, isLoading: false };
    setStoredAuth(newState);
    setState(newState);
  }, [state.token]);

  return (
    <AuthContext.Provider value={{ ...state, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
