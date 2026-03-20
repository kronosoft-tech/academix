import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Payment, CreatePaymentInput, UpdatePaymentInput } from "../../../shared/types/Payment";

interface BackendPaymentDto {
  id: string;
  student_id: string;
  group_id: string;
  amount: number;
  due_date: string;
  paid_date: string | null;
  status: string;
  method: string | null;
  reference: string | null;
}

interface UsePaymentsReturn {
  payments: Payment[];
  isLoading: boolean;
  error: string | null;
  createPayment: (data: CreatePaymentInput) => Promise<{ success: boolean; error?: string }>;
  updatePayment: (id: string, data: UpdatePaymentInput) => Promise<{ success: boolean; error?: string }>;
  deletePayment: (id: string) => Promise<{ success: boolean; error?: string }>;
  refetch: () => void;
}

function mapBackendToFrontend(dto: BackendPaymentDto): Payment {
  // Map backend status to frontend PaymentStatus
  const statusMap: Record<string, Payment["status"]> = {
    pending: "pending",
    paid: "completed",
    overdue: "failed",
    cancelled: "refunded",
    refunded: "refunded",
  };

  return {
    id: dto.id,
    studentId: dto.student_id,
    groupId: dto.group_id,
    amount: dto.amount,
    method: (dto.method as Payment["method"]) ?? "cash",
    status: statusMap[dto.status] ?? "pending",
    reference: dto.reference ?? undefined,
    paidAt: dto.paid_date ?? undefined,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

export function usePayments(): UsePaymentsReturn {
  const [payments, setPayments] = useState<Payment[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPayments = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendPaymentDto[] | null;
        error: string | null;
      }>("list_payments");

      if (response.success && response.data) {
        setPayments(response.data.map(mapBackendToFrontend));
      } else {
        setError(response.error || "Failed to fetch payments");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch payments");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchPayments();
  }, [fetchPayments]);

  const createPayment = async (data: CreatePaymentInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendPaymentDto | null;
        error: string | null;
      }>("create_payment", {
        request: {
          student_id: data.studentId,
          group_id: data.groupId,
          amount: data.amount,
          method: data.method,
        },
      });

      if (response.success) {
        await fetchPayments();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to create payment" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to create payment" };
    } finally {
      setIsLoading(false);
    }
  };

  const updatePayment = async (id: string, data: UpdatePaymentInput): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendPaymentDto | null;
        error: string | null;
      }>("update_payment", {
        id,
        request: {
          status: data.status ?? null,
          reference: data.reference ?? null,
          description: data.description ?? null,
          paid_date: data.paidAt ?? null,
        },
      });

      if (response.success) {
        await fetchPayments();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to update payment" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to update payment" };
    } finally {
      setIsLoading(false);
    }
  };

  const deletePayment = async (id: string): Promise<{ success: boolean; error?: string }> => {
    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        error: string | null;
      }>("delete_payment", { id });

      if (response.success) {
        await fetchPayments();
        return { success: true };
      } else {
        return { success: false, error: response.error || "Failed to delete payment" };
      }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : "Failed to delete payment" };
    } finally {
      setIsLoading(false);
    }
  };

  return {
    payments,
    isLoading,
    error,
    createPayment,
    updatePayment,
    deletePayment,
    refetch: fetchPayments,
  };
}
