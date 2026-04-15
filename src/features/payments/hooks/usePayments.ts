import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Payment, CreatePaymentInput, UpdatePaymentInput, PaymentStatusType } from "../../../shared/types/Payment";

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
  description: string | null;
  created_at: string;
  updated_at: string;
}

interface UsePaymentsReturn {
  payments: Payment[];
  isLoading: boolean;
  error: string | null;
  createPayment: (data: CreatePaymentInput) => Promise<{ success: boolean; error?: string }>;
  updatePayment: (id: string, data: UpdatePaymentInput) => Promise<{ success: boolean; error?: string }>;
  deletePayment: (id: string) => Promise<{ success: boolean; error?: string }>;
  refetch: () => void;
  getStudentPayments: (studentId: string) => Promise<Payment[]>;
  getStudentPaymentStatus: (studentId: string) => Promise<PaymentStatusType>;
}

function mapBackendToFrontend(dto: BackendPaymentDto): Payment {
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
    dueDate: dto.due_date ?? undefined,
    description: dto.description ?? undefined,
    createdAt: dto.created_at,
    updatedAt: dto.updated_at,
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
            description: data.description ?? null,
            paid: data.paid ?? null,
          },
        });

       if (response.success) {
         console.log("[FRONTEND] Payment created successfully, refetching...");
         await fetchPayments();
         return { success: true };
       } else {
         console.log("[FRONTEND] Payment creation failed:", response.error);
         return { success: false, error: response.error || "Failed to create payment" };
       }
     } catch (err) {
       console.log("[FRONTEND] Payment creation error:", err);
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

  const getStudentPayments = async (studentId: string): Promise<Payment[]> => {
    try {
      const response = await invoke<{
        success: boolean;
        data: BackendPaymentDto[] | null;
        error: string | null;
      }>("list_payments");
      
      if (response.success && response.data) {
        return response.data
          .filter(p => p.student_id === studentId)
          .map(mapBackendToFrontend);
      }
      return [];
    } catch {
      return [];
    }
  };

  const getStudentPaymentStatus = async (studentId: string): Promise<PaymentStatusType> => {
    const studentPayments = await getStudentPayments(studentId);
    
    if (studentPayments.length === 0) return "current";
    
    const now = new Date();
    const hasOverdue = studentPayments.some(p => 
      p.status === "pending" && p.dueDate && new Date(p.dueDate) < now
    );
    
    if (hasOverdue) return "delinquent";
    
    const hasPaid = studentPayments.some(p => p.status === "completed");
    if (hasPaid) return "current";
    
    return "current";
  };

  return {
    payments,
    isLoading,
    error,
    createPayment,
    updatePayment,
    deletePayment,
    refetch: fetchPayments,
    getStudentPayments,
    getStudentPaymentStatus,
  };
}
