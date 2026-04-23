// useEmployees Hook - Phase 6
// Handles employee CRUD operations

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  Employee,
  EmployeeListItem,
  EmployeeSummary,
  CreateEmployeeRequest,
  UpdateEmployeeRequest,
} from "../types";

export function useEmployees() {
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [summary, setSummary] = useState<EmployeeSummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // List employees
  const listEmployees = useCallback(async (filters?: {
    status?: string;
    department?: string;
    search?: string;
  }) => {
    setLoading(true);
    setError(null);
    try {
      const response = await invoke<{ success: boolean; data: EmployeeListItem[] | null; error: string | null }>("list_employees", {
        status: filters?.status,
        department: filters?.department,
        search: filters?.search,
      });
      if (response.success && response.data) {
        setEmployees(response.data.map(item => ({
          ...item,
          hire_date: new Date().toISOString(),
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          user_id: undefined,
          document_type: "dni" as const,
          first_name: item.full_name.split(" ")[0],
          last_name: item.full_name.split(" ").slice(1).join(" "),
          phone: undefined,
          address: undefined,
          bank_name: undefined,
          bank_account: undefined,
          account_type: undefined,
          cci: undefined,
          afp: undefined,
          termination_date: undefined,
        })));
        return response.data;
      } else {
        throw new Error(response.error || "Error listing employees");
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get employee by ID
  const getEmployee = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      return await invoke<Employee | null>("get_employee", { id });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Create employee
  const createEmployee = useCallback(async (request: CreateEmployeeRequest) => {
    setLoading(true);
    setError(null);
    try {
      const employee = await invoke<Employee>("create_employee", { request });
      setEmployees((prev) => [employee, ...prev]);
      return employee;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Update employee
  const updateEmployee = useCallback(async (id: string, request: UpdateEmployeeRequest) => {
    setLoading(true);
    setError(null);
    try {
      const employee = await invoke<Employee>("update_employee", { id, request });
      setEmployees((prev) =>
        prev.map((e) => (e.id === id ? employee : e))
      );
      return employee;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Delete employee
  const deleteEmployee = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      await invoke<void>("delete_employee", { id });
      setEmployees((prev) => prev.filter((e) => e.id !== id));
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // Get employee summary for dashboard
  const getSummary = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<EmployeeSummary>("get_employee_summary");
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
    employees,
    summary,
    loading,
    error,
    listEmployees,
    getEmployee,
    createEmployee,
    updateEmployee,
    deleteEmployee,
    getSummary,
    clearError,
  };
}