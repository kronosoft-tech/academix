export type PaymentStatus = "pending" | "completed" | "failed" | "refunded";
export type PaymentMethod = "cash" | "card" | "transfer" | "online";

export interface Payment {
  id: string;
  studentId: string;
  groupId: string;
  amount: number;
  method: PaymentMethod;
  status: PaymentStatus;
  reference?: string;
  description?: string;
  paidAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreatePaymentInput {
  studentId: string;
  groupId: string;
  amount: number;
  method: PaymentMethod;
  reference?: string;
  description?: string;
}

export interface UpdatePaymentInput {
  status?: PaymentStatus;
  reference?: string;
  description?: string;
  paidAt?: string;
}
