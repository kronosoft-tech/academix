import { useState, useEffect } from "react";
import type { Payment, PaymentStatusSummary, PaymentDelinquencyStatus } from "../../../shared/types/Payment";
import { Button } from "../../../shared/ui/components/Button";
import { Card } from "../../../shared/ui/components/Card";
import { usePayments } from "../hooks/usePayments";

interface StudentPaymentModalProps {
  studentId: string;
  studentName: string;
  groupId?: string;
  onClose: () => void;
}

interface StudentPaymentDetail {
  payments: Payment[];
  summary: PaymentStatusSummary | null;
}

export function StudentPaymentModal({ studentId, studentName, groupId = "", onClose }: StudentPaymentModalProps) {
  const { getStudentPaymentStatus, getStudentPayments } = usePayments();
  const [detail, setDetail] = useState<StudentPaymentDetail | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchDetail = async () => {
      setIsLoading(true);
      setError(null);
      
      try {
        // Get payment history for the student
        const payments = await getStudentPayments(studentId);
        
        // Get payment status - use provided groupId or empty string
        const summary = await getStudentPaymentStatus(studentId, groupId);
        
        setDetail({
          payments,
          summary,
        });
      } catch (err) {
        console.error("Error fetching payment details:", err);
        setError("Error al cargar los detalles de pago");
      } finally {
        setIsLoading(false);
      }
    };
    fetchDetail();
  }, [studentId, groupId, getStudentPaymentStatus, getStudentPayments]);

  // Calculate total paid amount
  const totalPaid = detail?.payments
    .filter(p => p.status === "completed")
    .reduce((sum, p) => sum + p.amount, 0) ?? 0;

  const getStatusLabel = (status: PaymentDelinquencyStatus) => {
    const labels = {
      current: "Al día",
      delinquent: "Atrasado",
      ahead: "Adelantado",
    };
    return labels[status];
  };

  const getStatusClass = (status: PaymentDelinquencyStatus) => {
    const classes = {
      current: "bg-green-100 text-green-800",
      delinquent: "bg-red-100 text-red-800",
      ahead: "bg-blue-100 text-blue-800",
    };
    return classes[status];
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b">
          <h2 className="text-xl font-bold text-gray-900">
            Estado de Pagos - {studentName}
          </h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-2xl"
          >
            &times;
          </button>
        </div>

        {/* Content */}
        <div className="p-6">
          {error && (
            <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
              {error}
            </div>
          )}
          
          {isLoading ? (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
              <p className="mt-2 text-gray-500">Cargando...</p>
            </div>
          ) : (detail && (detail.summary || detail.payments.length > 0)) ? (
            <div className="space-y-6">
              {/* Status Summary - only show if summary exists */}
              {detail.summary && (
                <>
                  <div className="grid grid-cols-2 gap-4">
                    <Card className="p-4">
                      <p className="text-sm text-gray-500">Estado</p>
                      <span
                        className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getStatusClass(
                          detail.summary.status
                        )}`}
                      >
                        {getStatusLabel(detail.summary.status)}
                      </span>
                    </Card>
                    <Card className="p-4">
                      <p className="text-sm text-gray-500">Días de atraso</p>
                      <p
                        className={`text-lg font-semibold ${
                          detail.summary.daysDelayed > 0
                            ? "text-red-600"
                            : detail.summary.daysDelayed < 0
                            ? "text-blue-600"
                            : "text-green-600"
                        }`}
                      >
                        {detail.summary.daysDelayed === 0
                          ? "Al día"
                          : detail.summary.daysDelayed > 0
                          ? `+${detail.summary.daysDelayed} días`
                          : `-${Math.abs(detail.summary.daysDelayed)} días`}
                      </p>
                    </Card>
                  </div>

                  {/* Payment Details - Course price, months paid, debt */}
                  <div className="grid grid-cols-3 gap-4">
                    <Card className="p-4">
                      <p className="text-sm text-gray-500">Precio Mensual</p>
                      <p className="text-lg font-semibold text-gray-900">
                        ${(detail.summary.coursePrice || 0).toLocaleString()}
                      </p>
                    </Card>
                    <Card className="p-4">
                      <p className="text-sm text-gray-500">Meses Pagados</p>
                      <p className="text-lg font-semibold text-gray-900">
                        {detail.summary.monthsPaid || 0}
                      </p>
                    </Card>
                    <Card className={`p-4 ${(detail.summary.debtAmount || 0) > 0 ? 'bg-red-50' : 'bg-green-50'}`}>
                      <p className="text-sm text-gray-500">Deuda</p>
                      <p className={`text-lg font-semibold ${(detail.summary.debtAmount || 0) > 0 ? 'text-red-600' : 'text-green-600'}`}>
                        ${(detail.summary.debtAmount || 0).toLocaleString()}
                      </p>
                    </Card>
                  </div>

                  {/* Group Info */}
                  <div className="bg-gray-50 rounded-lg p-4">
                    <p className="text-sm text-gray-500">Grupo</p>
                    <p className="text-lg font-medium text-gray-900">
                      {detail.summary.groupName}
                    </p>
                  </div>

                  {/* Due Date and Next Payment */}
                  <div className="grid grid-cols-2 gap-4">
                    <div className="bg-gray-50 rounded-lg p-4">
                      <p className="text-sm text-gray-500">Fecha de Inicio</p>
                      <p className="text-lg font-medium text-gray-900">
                        {detail.summary.dueDate ? new Date(detail.summary.dueDate).toLocaleDateString("es-CO", {
                          year: "numeric",
                          month: "long",
                          day: "numeric",
                        }) : "-"}
                      </p>
                    </div>
                    {detail.summary.nextPaymentDate && (
                      <div className="bg-blue-50 rounded-lg p-4">
                        <p className="text-sm text-blue-600">Próximo Pago</p>
                        <p className="text-lg font-semibold text-blue-800">
                          {new Date(detail.summary.nextPaymentDate).toLocaleDateString("es-CO", {
                            year: "numeric",
                            month: "long",
                            day: "numeric",
                          })}
                        </p>
                      </div>
                    )}
                  </div>
                </>
              )}

              {/* Total Paid */}
              <div className="bg-green-50 rounded-lg p-4">
                <p className="text-sm text-green-600">Total Pagado</p>
                <p className="text-2xl font-bold text-green-700">
                  ${totalPaid.toLocaleString()}
                </p>
                {detail.summary && detail.summary.monthsPaid && detail.summary.coursePrice > 0 && (
                  <p className="text-sm text-green-600 mt-1">
                    ({detail.summary.monthsPaid} meses x ${detail.summary.coursePrice.toLocaleString()})
                  </p>
                )}
              </div>

              {/* Payment History */}
              {detail.payments.length > 0 ? (
                <div>
                  <h3 className="text-lg font-semibold mb-3">Historial de Pagos</h3>
                  <div className="space-y-2">
                    {detail.payments.map((payment) => (
                      <div
                        key={payment.id}
                        className="flex justify-between items-center p-3 bg-white border rounded-lg"
                      >
                        <div>
                          <p className="font-medium">${payment.amount.toLocaleString()}</p>
                          <p className="text-sm text-gray-500">
                            Método: {payment.method === 'cash' ? 'Efectivo' : payment.method === 'card' ? 'Tarjeta' : payment.method === 'transfer' ? 'Transferencia' : 'Online'} | 
                            Estado: {payment.status === 'completed' ? 'Pagado' : payment.status === 'pending' ? 'Pendiente' : payment.status}
                          </p>
                          {payment.dueDate && (
                            <p className="text-xs text-gray-400">
                              Fecha límite: {new Date(payment.dueDate).toLocaleDateString('es-CO')}
                            </p>
                          )}
                        </div>
                        <div className="text-right">
                          {payment.paidAt && (
                            <p className="text-sm text-gray-500">
                              Pagado: {new Date(payment.paidAt).toLocaleDateString('es-CO')}
                            </p>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="text-center py-4 bg-gray-50 rounded-lg">
                  <p className="text-gray-500">No hay pagos registrados</p>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center py-8">
              <p className="text-gray-500">No se encontró información de pagos</p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-6 border-t bg-gray-50">
          <Button onClick={onClose}>Cerrar</Button>
        </div>
      </div>
    </div>
  );
}