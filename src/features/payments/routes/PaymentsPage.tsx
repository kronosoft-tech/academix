import { useState } from "react";
import { usePayments } from "../hooks/usePayments";
import { useStudents } from "../../students/hooks/useStudents";
import { useGroups } from "../../groups/hooks/useGroups";
import { useCourses } from "../../courses/hooks/useCourses";
import { useAuth } from "../../../shared/hooks/useAuth";
import { Card } from "../../../shared/ui/components/Card";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { SearchableSelect } from "../../../shared/ui/components/SearchableSelect";
import { Modal } from "../../../shared/ui/components/Modal";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type PaymentAny = any;

interface PaymentRowProps {
  payment: PaymentAny;
  students: PaymentAny[];
  onEdit: (payment: PaymentAny) => void;
  onDelete: (paymentId: string) => void;
  canEdit: boolean;
}

function PaymentRow({ payment, students, onEdit, onDelete, canEdit }: PaymentRowProps) {
  const isPaid = payment.status === "completed";
  const student = students.find(s => s.id === payment.studentId);
  
  // Handle case where payment data might be incomplete
  if (!payment || !payment.id) {
    return null;
  }
  
  return (
    <tr key={payment.id} className="hover:bg-[var(--color-foreground)]/5">
      <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]">
        {payment.reference || (payment.id ? payment.id.substring(0, 8) + "..." : "-")}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]">
        {student?.name || (payment.studentId ? payment.studentId.substring(0, 8) + "..." : "Sin estudiante")}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]">
        ${(payment.amount || 0).toLocaleString()}
      </td>
      <td className="px-6 py-4 whitespace-nowrap">
        <span
          className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
            isPaid
              ? "bg-green-100 text-green-800"
              : (payment.status === "pending")
              ? "bg-yellow-100 text-yellow-800"
              : (payment.status === "failed")
              ? "bg-red-100 text-red-800"
              : "bg-[var(--color-foreground)]/10 text-[var(--color-foreground)]"
          }`}
        >
          {isPaid
            ? "Pagado"
            : (payment.status === "pending")
            ? "Pendiente"
            : (payment.status === "failed")
            ? "Fallido"
            : "Reembolsado"}
        </span>
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]/60">
        {payment.method || "-"}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]/60">
        {payment.paidAt ? new Date(payment.paidAt).toLocaleDateString() : "-"}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]/60">
        {payment.description || "-"}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]/60">
        {payment.createdAt ? new Date(payment.createdAt).toLocaleDateString() : "-"}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm">
        {canEdit && (
          <div className="flex gap-2">
            <Button size="sm" variant="secondary" onClick={() => onEdit(payment)}>
              Editar
            </Button>
            <Button size="sm" variant="danger" onClick={() => onDelete(payment.id)}>
              Eliminar
            </Button>
          </div>
        )}
      </td>
    </tr>
  );
}

export default function PaymentsPage() {
  const { payments, isLoading, error, createPayment, updatePayment, deletePayment } = usePayments();
  const { students } = useStudents();
  const { groups } = useGroups();
  const { courses } = useCourses();
  const { user } = useAuth();
  
  const canEdit = user?.role === "admin" || user?.role === "empleado" || user?.role === "gerente";
  
  const [showForm, setShowForm] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [editingPayment, setEditingPayment] = useState<PaymentAny>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [formData, setFormData] = useState({
    studentId: "",
    groupId: "",
    amount: 0,
    method: "cash" as "cash" | "card" | "transfer" | "online",
    description: "",
  });
  const [editFormData, setEditFormData] = useState({
    status: "" as "pending" | "completed" | "failed" | "refunded",
    amount: 0,
    method: "cash" as "cash" | "card" | "transfer" | "online",
    reference: "",
    description: "",
    paidAt: "",
  });
  const [submitError, setSubmitError] = useState<string | null>(null);

  // Get course price from selected group
  const selectedGroup = groups.find(g => g.id === formData.groupId);
  const selectedCourse = selectedGroup ? courses.find(c => c.id === selectedGroup.courseId) : null;

  const filteredPayments = searchTerm 
    ? payments.filter((payment) =>
        payment.reference?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        payment.description?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        payment.id.toLowerCase().includes(searchTerm.toLowerCase())
      )
    : payments;

  // Auto-fill amount when group is selected
  const handleGroupChange = (groupId: string) => {
    const group = groups.find(g => g.id === groupId);
    const course = group ? courses.find(c => c.id === group.courseId) : null;
    setFormData({ 
      ...formData, 
      groupId, 
      amount: course?.price || 0 
    });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError(null);

    if (!formData.studentId) {
      setSubmitError("Debes seleccionar un estudiante");
      return;
    }
    if (!formData.groupId) {
      setSubmitError("Debes seleccionar un grupo");
      return;
    }
    if (formData.amount <= 0) {
      setSubmitError("El monto debe ser mayor a 0");
      return;
    }

    const result = await createPayment({
      studentId: formData.studentId,
      groupId: formData.groupId,
      amount: formData.amount,
      method: formData.method,
      description: formData.description || undefined,
      paid: true,
    });

    if (result.success) {
      setShowForm(false);
      setFormData({ studentId: "", groupId: "", amount: 0, method: "cash", description: "" });
    } else {
      setSubmitError(result.error || "Error al crear payment");
    }
  };

  const handleEdit = (payment: PaymentAny) => {
    setEditingPayment(payment);
    setEditFormData({
      status: payment.status,
      amount: payment.amount,
      method: payment.method || "cash",
      reference: payment.reference || "",
      description: payment.description || "",
      paidAt: payment.paidAt ? payment.paidAt.split("T")[0] : "",
    });
    setShowEditModal(true);
  };

  const handleEditSubmit = async () => {
    if (!editingPayment) return;
    
    const updateData: any = {
      status: editFormData.status === "completed" ? "paid" : editFormData.status,
      reference: editFormData.reference || null,
      description: editFormData.description || null,
      paidAt: editFormData.status === "completed" ? (editFormData.paidAt || new Date().toISOString()) : null,
    };
    
    const result = await updatePayment(editingPayment.id, updateData);
    
    if (result.success) {
      setShowEditModal(false);
      setEditingPayment(null);
    } else {
      setSubmitError(result.error || "Error al actualizar payment");
    }
  };

  const handleDelete = async (paymentId: string) => {
    if (!confirm("¿Estás seguro de eliminar este pago?")) return;
    
    const result = await deletePayment(paymentId);
    if (!result.success) {
      alert("Error: " + (result.error || "No se pudo eliminar"));
    }
  };

  const generatePaymentReceipt = (payment: PaymentAny) => {
    const student = students.find(s => s.id === payment.studentId);
    const group = groups.find(g => g.id === payment.groupId);
    const course = group ? courses.find(c => c.id === group.courseId) : null;
    
    const receiptContent = `
RECIBO DE PAGO
=============

No. Referencia: ${payment.reference || "N/A"}
Fecha: ${payment.createdAt ? new Date(payment.createdAt).toLocaleDateString() : "N/A"}

ESTUDIANTE:
Nombre: ${student?.name || "N/A"}
Documento: ${student?.documentNumber || "N/A"}

CURSO/GRUPO:
Curso: ${course?.name || "N/A"}
Grupo: ${group?.name || "N/A"}

DETALLE DEL PAGO:
Monto: $${payment.amount?.toLocaleString() || 0}
Método: ${payment.method || "N/A"}
Estado: ${payment.status === "completed" ? "PAGADO" : payment.status.toUpperCase()}
${payment.paidAt ? `Fecha de pago: ${new Date(payment.paidAt).toLocaleDateString()}` : ""}
${payment.description ? `Descripción: ${payment.description}` : ""}

===========================
Academix - Sistema de Gestión
    `.trim();

    // Create and download text file
    const blob = new Blob([receiptContent], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `recibo-${payment.reference || payment.id}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  if (isLoading && payments.length === 0) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Pagos</h1>
        <Button onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancelar" : "Nuevo Pago"}
        </Button>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
          {error}
        </div>
      )}

      {showForm && (
        <Card className="mb-6">
          <h2 className="text-lg font-semibold mb-4">Registrar Nuevo Pago</h2>
          <form onSubmit={handleSubmit} className="space-y-4">
            <SearchableSelect
              label="Estudiante"
              required
              placeholder="Buscar por nombre, apellido o ID..."
              value={formData.studentId}
              onChange={(id) => setFormData({ ...formData, studentId: id })}
              options={students}
              searchFields={["name", "id", "documentNumber"] as (keyof typeof students[0])[]}
              displayFormatter={(student) => `${student.name} - ${student.documentNumber}`}
              getItemValue={(student) => student.id}
              notFoundMessage="No se encontraron estudiantes"
            />

            <SearchableSelect
              label="Grupo"
              required
              placeholder="Buscar grupo..."
              value={formData.groupId}
              onChange={handleGroupChange}
              options={groups}
              searchFields={["name", "id"] as (keyof typeof groups[0])[]}
              displayFormatter={(group) => {
                const course = courses.find(c => c.id === group.courseId);
                return `${group.name} ${course ? `($${course.price.toLocaleString()})` : ""}`;
              }}
              getItemValue={(group) => group.id}
              notFoundMessage="No se encontraron grupos"
            />

            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
                Monto {selectedCourse && `(del curso: $${selectedCourse.price.toLocaleString()})`}
              </label>
              <Input
                type="number"
                placeholder="0.00"
                value={formData.amount}
                onChange={(e) => setFormData({ ...formData, amount: parseFloat(e.target.value) || 0 })}
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">Método de Pago</label>
              <select
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                value={formData.method}
                onChange={(e) => setFormData({ ...formData, method: e.target.value as typeof formData.method })}
              >
                <option value="cash">Efectivo</option>
                <option value="card">Tarjeta</option>
                <option value="transfer">Transferencia</option>
                <option value="online">Pago en línea</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
                Descripción (opcional)
              </label>
              <textarea
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="Notas adicionales sobre el pago..."
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={2}
              />
            </div>

            {submitError && (
              <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">
                {submitError}
              </div>
            )}

            <div className="flex gap-2">
              <Button type="submit" loading={isLoading}>
                Registrar y Cobrar
              </Button>
              <Button type="button" variant="secondary" onClick={() => setShowForm(false)}>
                Cancelar
              </Button>
            </div>
          </form>
        </Card>
      )}

      <div className="mb-4">
        <Input
          placeholder="Buscar pagos por referencia o descripción..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      {filteredPayments.length === 0 ? (
        <Card className="text-center py-12">
          <p className="text-[var(--color-foreground)]/60">No hay pagos registrados</p>
          <Button className="mt-4" onClick={() => setShowForm(true)}>
            Registrar Primer Pago
          </Button>
        </Card>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-[var(--color-foreground)]/5">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Referencia
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Estudiante
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Monto
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Estado
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Método
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Fecha Pago
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Descripción
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Creado
                </th>
                {canEdit && (
                  <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                    Acciones
                  </th>
                )}
              </tr>
            </thead>
            <tbody className="bg-[var(--color-background)] divide-y divide-gray-200">
              {filteredPayments.map((payment) => (
                <PaymentRow 
                  key={payment.id} 
                  payment={payment} 
                  students={students}
                  onEdit={handleEdit}
                  onDelete={handleDelete}
                  canEdit={canEdit}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Edit Modal */}
      <Modal isOpen={showEditModal} onClose={() => setShowEditModal(false)} title="Editar Pago">
        <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">Estado</label>
              <select
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg"
                value={editFormData.status}
                onChange={(e) => setEditFormData({ ...editFormData, status: e.target.value as any })}
              >
                <option value="pending">Pendiente</option>
                <option value="completed">Pagado</option>
                <option value="failed">Fallido</option>
                <option value="refunded">Reembolsado</option>
              </select>
            </div>
            
            <Input
              label="Monto"
              type="number"
              value={editFormData.amount}
              onChange={(e) => setEditFormData({ ...editFormData, amount: parseFloat(e.target.value) || 0 })}
            />
            
            <Input
              label="Referencia"
              value={editFormData.reference}
              onChange={(e) => setEditFormData({ ...editFormData, reference: e.target.value })}
            />
            
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">Método</label>
              <select
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg"
                value={editFormData.method}
                onChange={(e) => setEditFormData({ ...editFormData, method: e.target.value as any })}
              >
                <option value="cash">Efectivo</option>
                <option value="card">Tarjeta</option>
                <option value="transfer">Transferencia</option>
                <option value="online">Pago en línea</option>
              </select>
            </div>
            
            {editFormData.status === "completed" && (
              <Input
                label="Fecha de pago"
                type="date"
                value={editFormData.paidAt}
                onChange={(e) => setEditFormData({ ...editFormData, paidAt: e.target.value })}
              />
            )}
            
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">Descripción</label>
              <textarea
                className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg"
                value={editFormData.description}
                onChange={(e) => setEditFormData({ ...editFormData, description: e.target.value })}
                rows={2}
              />
            </div>

            <div className="flex gap-2">
              <Button onClick={handleEditSubmit} loading={isLoading}>
                Guardar Cambios
              </Button>
              <Button variant="secondary" onClick={() => setShowEditModal(false)}>
                Cancelar
              </Button>
              <Button variant="primary" onClick={() => generatePaymentReceipt(editingPayment)}>
                Descargar Recibo
              </Button>
            </div>
          </div>
      </Modal>
    </div>
  );
}