import { useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ApiState<T> {
  data: T | null;
  error: string | null;
  isLoading: boolean;
}

interface UseApiOptions {
  onSuccess?: (data: unknown) => void;
  onError?: (error: string) => void;
}

export function useApi<T>(command: string, options?: UseApiOptions) {
  const [state, setState] = useState<ApiState<T>>({
    data: null,
    error: null,
    isLoading: false,
  });

  // Use ref to avoid stale closure for callbacks
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const execute = useCallback(
    async (params?: Record<string, unknown>) => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        const result = await invoke<T>(command, params);
        setState({ data: result, error: null, isLoading: false });
        optionsRef.current?.onSuccess?.(result);
        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setState({ data: null, error: errorMessage, isLoading: false });
        optionsRef.current?.onError?.(errorMessage);
        throw err;
      }
    },
    [command]
  );

  const reset = useCallback(() => {
    setState({ data: null, error: null, isLoading: false });
  }, []);

  return {
    ...state,
    execute,
    reset,
  };
}
