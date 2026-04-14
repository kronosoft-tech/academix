import { useState } from "react";
import { usePayments } from "../hooks/usePayments";
import { useStudents } from "../../students/hooks/useStudents";
import { useGroups } from "../../groups/hooks/useGroups";
import { Card } from "../../../shared/ui/components/Card";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { SearchableSelect } from "../../../shared/ui/components/SearchableSelect";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type PaymentAny = any;

interface PaymentRowProps {
  payment: PaymentAny;
  onMarkPaid: (id: string) => void;
}

function PaymentRow({ payment, onMarkPaid }: PaymentRowProps) {
  const isPending = payment.status === "pending";
  
  return (
    <tr key={payment.id} className="hover:bg-gray-50">
      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
        {payment.id.substring(0, 8)}...
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
        ${payment.amount.toLocaleString()}
      </td>
      <td className="px-6 py-4 whitespace-nowrap">
        <span
          className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
            payment.status === "completed"
              ? "bg-green-100 text-green-800"
              : payment.status === "pending"
              ? "bg-yellow-100 text-yellow-800"
              : payment.status === "failed"
              ? "bg-red-100 text-red-800"
              : "bg-gray-100 text-gray-800"
          }`}
        >
          {payment.status === "completed"
            ? "Pagado"
            : payment.status === "pending"
            ? "Pendiente"
            : payment.status === "failed"
            ? "Fallido"
            : "Reembolsado"}
        </span>
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {payment.reference || "-"}
      </td>
      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
        {payment.method || "-"}
      </td>
      <td className="px-6 py-4 whitespace-nowrap">
        {isPending && (
          <Button 
            size="sm" 
            variant="primary"
            onClick={() => onMarkPaid(payment.id)}
          >
            Marcar Pagado
          </Button>
        )}
      </td>
    </tr>
  );
}

export default function PaymentsPage() {
  const { payments, isLoading, error, createPayment, updatePayment, refetch } = usePayments();
  const { students } = useStudents();
  const { groups } = useGroups();
  const [showForm, setShowForm] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [_updatingId, setUpdatingId] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    studentId: "",
    groupId: "",
    amount: 0,
    method: "cash" as "cash" | "card" | "transfer" | "online",
  });
  const [submitError, setSubmitError] = useState<string | null>(null);

  const filteredPayments = payments.filter((payment) =>
    payment.reference?.toLowerCase().includes(searchTerm.toLowerCase())
  );

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
    });

    if (result.success) {
      setShowForm(false);
      setFormData({ studentId: "", groupId: "", amount: 0, method: "cash" });
    } else {
      setSubmitError(result.error || "Error al crear payment");
    }
  };

  const handleMarkPaid = async (paymentId: string) => {
    setUpdatingId(paymentId);
    const result = await updatePayment(paymentId, {
      status: "completed",
      reference: `PAG-${paymentId.substring(0, 8)}`,
    });
    
    if (result.success) {
      alert("✓ Pago marcado como pagado y asiento contable creado automáticamente");
    } else {
      alert("Error: " + (result.error || "No se pudo marcar el pago"));
    }
    setUpdatingId(null);
  };
  
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  void _updatingId; // Placeholder to avoid unused warning

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
        <h1 className="text-2xl font-bold text-gray-900">Pagos</h1>
        <Button onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancelar" : "Nuevo Pago"}
        </Button>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {submitError && (
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
          {submitError}
        </div>
      )}

      {showForm && (
        <Card className="mb-6">
          <h2 className="text-lg font-semibold mb-4">Registrar Nuevo Pago</h2>
          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Buscador de Estudiante */}
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

            {/* Selector de Grupo */}
            <SearchableSelect
              label="Grupo"
              required
              placeholder="Buscar grupo..."
              value={formData.groupId}
              onChange={(id) => setFormData({ ...formData, groupId: id })}
              options={groups}
              searchFields={["name", "id"] as (keyof typeof groups[0])[]}
              displayFormatter={(group) => group.name}
              getItemValue={(group) => group.id}
              notFoundMessage="No se encontraron grupos"
            />

            <Input
              label="Monto"
              type="number"
              placeholder="0.00"
              value={formData.amount}
              onChange={(e) => setFormData({ ...formData, amount: parseFloat(e.target.value) || 0 })}
              required
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Método de Pago</label>
              <select
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                value={formData.method}
                onChange={(e) => setFormData({ ...formData, method: e.target.value as typeof formData.method })}
              >
                <option value="cash">Efectivo</option>
                <option value="card">Tarjeta</option>
                <option value="transfer">Transferencia</option>
                <option value="online">Pago en línea</option>
              </select>
            </div>

            <div className="flex gap-2">
              <Button type="submit" loading={isLoading}>Registrar Pago</Button>
              <Button type="button" variant="secondary" onClick={() => setShowForm(false)}>
                Cancelar
              </Button>
            </div>
          </form>
        </Card>
      )}

      <div className="mb-4">
        <Input
          placeholder="Buscar pagos..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>

      {filteredPayments.length === 0 ? (
        <Card className="text-center py-12">
          <p className="text-gray-500">No hay pagos registrados</p>
          <Button className="mt-4" onClick={() => setShowForm(true)}>
            Registrar Primer Pago
          </Button>
        </Card>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  ID
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Monto
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Estado
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Referencia
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Método
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Acción
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filteredPayments.map((payment) => (
                <PaymentRow 
                  key={payment.id} 
                  payment={payment as PaymentRowProps["payment"]} 
                  onMarkPaid={handleMarkPaid}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="mt-4">
        <Button variant="secondary" onClick={refetch}>
          Actualizar
        </Button>
      </div>
    </div>
  );
}
