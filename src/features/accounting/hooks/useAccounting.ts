// useAccounting Hook - Phase 6
// Handles accounting entries, accounts, trial balance, and income statement

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  AccountingEntry,
  AccountCategory,
  AccountCategoryTreeNode,
  TrialBalance,
  IncomeStatement,
  AccountingSummary,
  FinancialBalance,
  CreateEntryRequest,
  EntryFilters,
  AccountFilters,
  Liability,
  CreateLiabilityRequest,
  Equity,
  CreateEquityRequest,
  FixedAsset,
  CreateFixedAssetRequest,
} from "../types";

export function useAccounting() {
  const [entries, setEntries] = useState<AccountingEntry[]>([]);
  const [accounts, setAccounts] = useState<AccountCategory[]>([]);
  const [summary, setSummary] = useState<AccountingSummary | null>(null);
  const [liabilities, setLiabilities] = useState<Liability[]>([]);
  const [equities, setEquities] = useState<Equity[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Create accounting entry
  const createEntry = useCallback(async (request: CreateEntryRequest, createdBy = "system") => {
    setLoading(true);
    setError(null);
    try {
      const entry = await invoke<AccountingEntry>("create_entry", {
        request,
        createdBy,
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

  // List entries with filters
  const listEntries = useCallback(async (filters?: EntryFilters) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AccountingEntry[]>("list_entries", {
        dateFrom: filters?.date_from,
        dateTo: filters?.date_to,
        entryType: filters?.entry_type,
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

  // Get entry by ID
  const getEntry = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<AccountingEntry | null>("get_entry", { id });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // List accounts
  const listAccounts = useCallback(async (filters?: AccountFilters) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AccountCategory[]>("list_accounts", {
        categoryType: filters?.category_type,
        activeOnly: filters?.active_only ?? true,
      });
      setAccounts(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get account tree
  const getAccountTree = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<AccountCategoryTreeNode[]>("get_account_tree");
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get trial balance
  const getTrialBalance = useCallback(async (asOfDate: string) => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<TrialBalance>("get_trial_balance", { asOfDate });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get income statement
  const getIncomeStatement = useCallback(async (periodStart: string, periodEnd: string) => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<IncomeStatement>("get_income_statement", {
        periodStart,
        periodEnd,
      });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get accounting summary for dashboard
  const getSummary = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AccountingSummary>("get_accounting_summary");
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

  // Get financial balance
  const getFinancialBalance = useCallback(async (asOfDate: string) => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<FinancialBalance>("get_financial_balance", { asOfDate });
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

  // Get accounts (convenience wrapper)
  const getAccounts = useCallback(async () => {
    return listAccounts({ active_only: true });
  }, [listAccounts]);

  // === Pasivos / Liabilities ===
  const createLiability = useCallback(async (request: CreateLiabilityRequest) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<Liability>("create_liability", { request });
      setLiabilities(prev => [result, ...prev]);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const listLiabilities = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<Liability[]>("list_liabilities");
      setLiabilities(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const payLiability = useCallback(async (id: string, amount: number) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<Liability>("pay_liability", { id, amount });
      setLiabilities(prev => prev.map(l => l.id === id ? result : l));
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // === Patrimonio / Equity ===
  const createEquity = useCallback(async (request: CreateEquityRequest) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<Equity>("create_equity", { request });
      setEquities(prev => [result, ...prev]);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  const listEquities = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<Equity[]>("list_equities");
      setEquities(result);
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // === Fixed Assets / Activos Fijos ===
  const createFixedAsset = useCallback(async (request: CreateFixedAssetRequest) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<FixedAsset>("create_fixed_asset", { request });
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    entries,
    accounts,
    summary,
    liabilities,
    equities,
    loading,
    error,
    createEntry,
    listEntries,
    getEntry,
    listAccounts,
    getAccounts,
    getAccountTree,
    getTrialBalance,
    getIncomeStatement,
    getSummary,
    getFinancialBalance,
    clearError,
    // Pasivos
    createLiability,
    listLiabilities,
    payLiability,
    // Patrimonio
    createEquity,
    listEquities,
    // Activos Fijos
    createFixedAsset,
  };
}