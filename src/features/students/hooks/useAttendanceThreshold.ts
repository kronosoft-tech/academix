import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ThresholdResponse {
  success: boolean;
  data: { value: number } | null;
  error: string | null;
}

interface UseAttendanceThresholdReturn {
  threshold: number;
  updateThreshold: (value: number) => Promise<{ success: boolean; error?: string }>;
  isLoading: boolean;
}

export function useAttendanceThreshold(): UseAttendanceThresholdReturn {
  const [threshold, setThreshold] = useState(3);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    invoke<ThresholdResponse>("get_absence_threshold").then((res) => {
      if (res.success && res.data) {
        setThreshold(res.data.value);
      }
      setIsLoading(false);
    }).catch(() => {
      setIsLoading(false);
    });
  }, []);

  const updateThreshold = useCallback(async (value: number) => {
    const validated = Math.max(1, Math.min(30, value));
    try {
      const res = await invoke<ThresholdResponse>("set_absence_threshold", {
        value: validated,
      });
      if (res.success && res.data) {
        setThreshold(res.data.value);
        return { success: true };
      }
      return { success: false, error: res.error || "Error al guardar umbral" };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : "Error al guardar umbral",
      };
    }
  }, []);

  return { threshold, updateThreshold, isLoading };
}
