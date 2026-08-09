export type PaymentStatus = "pending" | "completed" | "failed" | "refunded";
export type PaymentMethod = "cash" | "card" | "transfer" | "online";
export type PaymentType = "enrollment" | "tuition";
export type PaymentStatusType = "current" | "delinquent" | "ahead";
export type PaymentDelinquencyStatus = "current" | "delinquent" | "ahead";

export interface PaymentStatusSummary {
  studentId: string;
  studentName: string;
  groupName: string;
  status: PaymentStatusType;
  daysDelayed: number;
  dueDate?: string;
  coursePrice?: number;
  monthsPaid?: number;
  debtAmount?: number;
  nextPaymentDate?: string;
}

export interface Payment {
  id: string;
  studentId: string;
  groupId: string;
  amount: number;
  method: PaymentMethod;
  paymentType: PaymentType;
  status: PaymentStatus;
  reference?: string;
  description?: string;
  paidAt?: string;
  dueDate?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreatePaymentInput {
  studentId: string;
  groupId: string;
  amount: number;
  method: PaymentMethod;
  paymentType?: PaymentType;
  reference?: string;
  description?: string;
  paid?: boolean; // If true, payment is created as "paid" immediately
}

export interface UpdatePaymentInput {
  status?: PaymentStatus;
  reference?: string;
  description?: string;
  paidAt?: string;
}
