import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface DashboardStats {
  totalStudents: number;
  totalCourses: number;
  totalGroups: number;
  totalPayments: number;
  pendingPayments: number;
  attendanceRate: number;
}

interface UseDashboardReturn {
  stats: DashboardStats | null;
  isLoading: boolean;
  error: string | null;
  refetch: () => void;
}

export function useDashboard(): UseDashboardReturn {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStats = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      // Fetch all counts in parallel
      const [studentsRes, coursesRes, groupsRes, paymentsRes] = await Promise.all([
        invoke<{ success: boolean; data: unknown[] | null }>("list_students"),
        invoke<{ success: boolean; data: unknown[] | null }>("list_courses"),
        invoke<{ success: boolean; data: unknown[] | null }>("list_groups"),
        invoke<{ success: boolean; data: unknown[] | null }>("list_payments"),
      ]);

      const totalStudents = studentsRes.success && studentsRes.data ? studentsRes.data.length : 0;
      const totalCourses = coursesRes.success && coursesRes.data ? coursesRes.data.length : 0;
      const totalGroups = groupsRes.success && groupsRes.data ? groupsRes.data.length : 0;
      const totalPayments = paymentsRes.success && paymentsRes.data ? paymentsRes.data.length : 0;

      // Calculate pending payments
      const pendingPayments =
        paymentsRes.success && paymentsRes.data
          ? (paymentsRes.data as Array<{ status: string }>).filter((p) => p.status === "pending").length
          : 0;

      setStats({
        totalStudents,
        totalCourses,
        totalGroups,
        totalPayments,
        pendingPayments,
        attendanceRate: 0, // Would need a separate query for this
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch dashboard stats");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  return {
    stats,
    isLoading,
    error,
    refetch: fetchStats,
  };
}
