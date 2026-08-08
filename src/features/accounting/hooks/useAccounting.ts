// useAccounting Hook - Simplified
// Handles income/expense CRUD and summary

import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { appEvents, APP_EVENTS } from "../../../shared/utils/appEvents";
import type {
  AccountingEntry,
  AccountingSummary,
  CreateEntryPayload,
  EntryFilters,
} from "../types";

export function useAccounting() {
  const [entries, setEntries] = useState<AccountingEntry[]>([]);
  const [summary, setSummary] = useState<AccountingSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastFilters, setLastFilters] = useState<EntryFilters | undefined>();

  const createIncomeEntry = useCallback(async (payload: CreateEntryPayload) => {
    setLoading(true);
    setError(null);
    try {
      const entry = await invoke<AccountingEntry>("create_entry", {
        request: {
          date: payload.date,
          entry_type: "income",
          category: payload.category,
          description: payload.description,
          amount: payload.amount,
        },
      });
      setEntries((prev) => [entry, ...prev]);
      return entry;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const createExpenseEntry = useCallback(async (payload: CreateEntryPayload) => {
    setLoading(true);
    setError(null);
    try {
      const entry = await invoke<AccountingEntry>("create_entry", {
        request: {
          date: payload.date,
          entry_type: "expense",
          category: payload.category,
          description: payload.description,
          amount: payload.amount,
        },
      });
      setEntries((prev) => [entry, ...prev]);
      return entry;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const listEntries = useCallback(async (filters?: EntryFilters) => {
    setLoading(true);
    setError(null);
    setLastFilters(filters);
    try {
      const result = await invoke<AccountingEntry[]>("list_entries", {
        date_from: filters?.date_from,
        date_to: filters?.date_to,
        entry_type: filters?.type,
      });
      setEntries(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const getSummary = useCallback(async (dateFrom?: string, dateTo?: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AccountingSummary>("get_accounting_summary", {
        date_from: dateFrom,
        date_to: dateTo,
      });
      console.log("[ACCOUNTING] getSummary result:", JSON.stringify({ income: result.total_income, expenses: result.total_expenses, entries: result.entry_count, monthly: result.monthly_data?.length }));
      setSummary(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      console.error("[ACCOUNTING] getSummary ERROR:", message);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const deleteEntry = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>("delete_entry", { id });
      setEntries((prev) => prev.filter((e) => e.id !== id));
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  // Listen for cross-module data changes (e.g., payment created auto-income)
  useEffect(() => {
    const handleChange = () => {
      if (lastFilters) {
        listEntries(lastFilters);
        if (lastFilters.date_from && lastFilters.date_to) {
          getSummary(lastFilters.date_from, lastFilters.date_to);
        }
      }
    };

    appEvents.on(APP_EVENTS.ACCOUNTING_CHANGED, handleChange);
    return () => {
      appEvents.off(APP_EVENTS.ACCOUNTING_CHANGED, handleChange);
    };
  }, [lastFilters, listEntries, getSummary]);

  return {
    entries,
    summary,
    loading,
    error,
    createIncomeEntry,
    createExpenseEntry,
    listEntries,
    getSummary,
    deleteEntry,
    clearError,
  };
}
