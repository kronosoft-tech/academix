// usePayroll Hook - Phase 6
// Handles payroll runs and entries

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  PayrollRun,
  PayrollEntry,
  PayrollRunWithEntries,
  PayrollSummary,
  RunPayrollRequest,
  PayrollFilters,
} from "../types";

export function usePayroll() {
  const [runs, setRuns] = useState<PayrollRun[]>([]);
  const [currentRun, setCurrentRun] = useState<PayrollRunWithEntries | null>(null);
  const [summary, setSummary] = useState<PayrollSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // List payroll runs
  const listRuns = useCallback(async (filters?: PayrollFilters) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<PayrollRun[]>("list_payroll_runs", {
        periodStart: filters?.period_start,
        periodEnd: filters?.period_end,
        status: filters?.status,
      });
      setRuns(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get payroll run with entries
  const getRunWithEntries = useCallback(async (runId: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<PayrollRunWithEntries>("get_payroll_run", {
        runId,
      });
      setCurrentRun(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Run payroll
  const runPayroll = useCallback(async (request: RunPayrollRequest) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<PayrollRun>("run_payroll", { request });
      setRuns((prev) => [result, ...prev]);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Process payroll entry
  const processEntry = useCallback(async (entryId: string) => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<PayrollEntry>("process_payroll_entry", {
        entryId,
      });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Mark entry as paid
  const markAsPaid = useCallback(async (entryId: string) => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<PayrollEntry>("mark_payroll_entry_paid", {
        entryId,
      });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Cancel payroll run
  const cancelRun = useCallback(async (runId: string) => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>("cancel_payroll_run", { runId });
      setRuns((prev) =>
        prev.map((r) =>
          r.id === runId ? { ...r, status: "cancelled" as const } : r
        )
      );
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get payroll summary
  const getSummary = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<PayrollSummary>("get_payroll_summary");
      setSummary(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Clear error
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    runs,
    currentRun,
    summary,
    loading,
    error,
    listRuns,
    getRunWithEntries,
    runPayroll,
    processEntry,
    markAsPaid,
    cancelRun,
    getSummary,
    clearError,
  };
}