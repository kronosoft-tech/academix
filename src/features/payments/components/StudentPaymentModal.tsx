import { useState, useEffect } from "react";
import type { Payment, PaymentStatusType } from "../../../shared/types/Payment";
import { Button } from "../../../shared/ui/components/Button";
import { Card } from "../../../shared/ui/components/Card";
import { usePayments } from "../hooks/usePayments";

interface StudentPaymentModalProps {
  studentId: string;
  studentName: string;
  onClose: () => void;
}

interface StudentPaymentDetail {
  payments: Payment[];
  status: PaymentStatusType;
}

export function StudentPaymentModal({ studentId, studentName, onClose }: StudentPaymentModalProps) {
  const { getStudentPaymentStatus, getStudentPayments } = usePayments();
  const [detail, setDetail] = useState<StudentPaymentDetail | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchDetail = async () => {
      setIsLoading(true);
      setError(null);
      
      try {
        const payments = await getStudentPayments(studentId);
        const status = await getStudentPaymentStatus(studentId);
        
        setDetail({
          payments,
          status,
        });
      } catch (err) {
        console.error("Error fetching payment details:", err);
        setError("Error al cargar los detalles de pago");
      } finally {
        setIsLoading(false);
      }
    };
    fetchDetail();
  }, [studentId, getStudentPaymentStatus, getStudentPayments]);

  // Calculate totals
  const totalPaid = detail?.payments
    .filter(p => p.status === "completed")
    .reduce((sum, p) => sum + p.amount, 0) ?? 0;

  const totalPending = detail?.payments
    .filter(p => p.status === "pending")
    .reduce((sum, p) => sum + p.amount, 0) ?? 0;

  const getStatusLabel = (status: PaymentStatusType) => {
    const labels = {
      current: "Al día",
      delinquent: "Atrasado",
      ahead: "Adelantado",
    };
    return labels[status] || status;
  };

  const getStatusClass = (status: PaymentStatusType) => {
    const classes = {
      current: "bg-green-100 text-green-800",
      delinquent: "bg-red-100 text-red-800",
      ahead: "bg-blue-100 text-blue-800",
    };
    return classes[status] || "bg-gray-100 text-gray-800";
  };

  if (isLoading) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div className="bg-white rounded-lg p-6">
          <p className="text-gray-600">Cargando...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div className="bg-white rounded-lg p-6">
          <p className="text-red-600">{error}</p>
          <Button onClick={onClose} className="mt-4">Cerrar</Button>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <div className="p-6">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-xl font-bold text-gray-900">
              Pagos de {studentName}
            </h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600"
            >
              ✕
            </button>
          </div>

          {detail && (
            <>
              {/* Status Badge */}
              <div className="mb-6">
                <span className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getStatusClass(detail.status)}`}>
                  {getStatusLabel(detail.status)}
                </span>
              </div>

              {/* Summary Cards */}
              <div className="grid grid-cols-2 gap-4 mb-6">
                <Card className="p-4">
                  <p className="text-sm text-gray-500">Total Pagado</p>
                  <p className="text-2xl font-bold text-green-600">
                    ${totalPaid.toLocaleString("es-CO")}
                  </p>
                </Card>
                <Card className="p-4">
                  <p className="text-sm text-gray-500">Total Pendiente</p>
                  <p className="text-2xl font-bold text-orange-600">
                    ${totalPending.toLocaleString("es-CO")}
                  </p>
                </Card>
              </div>

              {/* Payment History */}
              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-4">
                  Historial de Pagos
                </h3>
                {detail.payments.length === 0 ? (
                  <p className="text-gray-500">No hay pagos registrados</p>
                ) : (
                  <div className="space-y-3">
                    {detail.payments.map((payment) => (
                      <div
                        key={payment.id}
                        className="flex justify-between items-center p-3 bg-gray-50 rounded-lg"
                      >
                        <div>
                          <p className="font-medium text-gray-900">
                            ${payment.amount.toLocaleString("es-CO")}
                          </p>
                          <p className="text-sm text-gray-500">
                            {payment.method} - {payment.status}
                          </p>
                        </div>
                        <div className="text-right">
                          {payment.paidAt && (
                            <p className="text-sm text-gray-500">
                              Pagado: {new Date(payment.paidAt).toLocaleDateString("es-CO")}
                            </p>
                          )}
                          {payment.dueDate && (
                            <p className="text-sm text-orange-500">
                              Vence: {new Date(payment.dueDate).toLocaleDateString("es-CO")}
                            </p>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </>
          )}

          <div className="mt-6 flex justify-end">
            <Button onClick={onClose}>Cerrar</Button>
          </div>
        </div>
      </div>
    </div>
  );
}
