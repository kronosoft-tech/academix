import { useState } from "react";
import { useStudents } from "../hooks/useStudents";
import { useGroups } from "../../groups/hooks/useGroups";
import { useCourses } from "../../courses/hooks/useCourses";
import { usePayments } from "../../payments/hooks/usePayments";
import { Card } from "../../../shared/ui/components/Card";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { SearchableSelect } from "../../../shared/ui/components/SearchableSelect";
import { Modal } from "../../../shared/ui/components/Modal";
import type { Student } from "../../../shared/types/Student";

interface PaymentStatus {
  isPaid: boolean;
  daysOverdue: number;
  nextPaymentDate: string;
  lastPaymentDate: string;
  monthlyAmount: number;
  totalPaid: number;
  pendingAmount: number;
}

// Format date from RFC3339 to YYYY-MM-DD for display
function formatDate(dateStr: string | undefined): string {
  if (!dateStr) return "-";
  return dateStr.split("T")[0];
}

export default function StudentsPage() {
  const { students, isLoading, error, createStudent, updateStudent, deleteStudent, refetch } = useStudents();
  const { groups } = useGroups();
  const { courses } = useCourses();
  const { payments } = usePayments();
  
  const [searchTerm, setSearchTerm] = useState("");
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingStudent, setEditingStudent] = useState<Student | null>(null);
  const [selectedStudent, setSelectedStudent] = useState<Student | null>(null);
  const [studentPaymentStatus, setStudentPaymentStatus] = useState<PaymentStatus | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    documentNumber: "",
    documentType: "cc" as "cc" | "ti" | "ce" | "rc" | "nip",
    email: "",
    phone: "",
    address: "",
    birthDate: "",
    guardianName: "",
    guardianDocument: "",
    guardianPhone: "",
    courseId: "",
    groupId: "",
  });

  const isMinor = (birthDate: string): boolean => {
    if (!birthDate) return false;
    const birth = new Date(birthDate);
    const today = new Date();
    let age = today.getFullYear() - birth.getFullYear();
    const monthDiff = today.getMonth() - birth.getMonth();
    if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birth.getDate())) {
      age--;
    }
    return age < 18;
  };

  const showGuardianWarning = formData.birthDate && isMinor(formData.birthDate) && 
    (!formData.guardianName || !formData.guardianPhone);

  // Calculate payment status for a student
  const calculatePaymentStatus = (studentId: string): PaymentStatus => {
    const studentPayments = payments.filter(p => p.studentId === studentId);
    const totalPaid = studentPayments
      .filter(p => p.status === "completed")
      .reduce((sum, p) => sum + p.amount, 0);
    
    const student = students.find(s => s.id === studentId);
    const course = student?.courseId ? courses.find(c => c.id === student.courseId) : null;
    const monthlyAmount = course?.price || 200000;
    
    const monthsActive = Math.max(1, Math.ceil((Date.now() - new Date(studentPayments[0]?.createdAt || Date.now()).getTime()) / (30 * 24 * 60 * 60 * 1000)));
    const expectedPayment = monthlyAmount * monthsActive;
    const pendingAmount = Math.max(0, expectedPayment - totalPaid);
    
    // Calculate days overdue
    const lastPayment = studentPayments
      .filter(p => p.status === "completed")
      .sort((a, b) => new Date(b.paidAt || b.createdAt).getTime() - new Date(a.paidAt || a.createdAt).getTime())[0];
    
    const daysSinceLastPayment = lastPayment 
      ? Math.floor((Date.now() - new Date(lastPayment.paidAt || lastPayment.createdAt).getTime()) / (24 * 60 * 60 * 1000))
      : 30;
    
    const daysOverdue = Math.max(0, daysSinceLastPayment - 30);
    
    // Calculate next payment date
    const nextPaymentDate = lastPayment
      ? new Date(new Date(lastPayment.paidAt || lastPayment.createdAt).getTime() + 30 * 24 * 60 * 60 * 1000).toISOString().split("T")[0]
      : new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString().split("T")[0];

    return {
      isPaid: pendingAmount === 0,
      daysOverdue,
      nextPaymentDate,
      lastPaymentDate: lastPayment?.paidAt?.split("T")[0] || "Sin pagos",
      monthlyAmount,
      totalPaid,
      pendingAmount,
    };
  };

  // Filter students based on search term
  const filteredStudents = students.filter(student => {
    if (!searchTerm) return true;
    const term = searchTerm.toLowerCase();
    return (
      student.name.toLowerCase().includes(term) ||
      student.id.toLowerCase().includes(term) ||
      student.documentNumber.toLowerCase().includes(term) ||
      student.email.toLowerCase().includes(term) ||
      (student.phone && student.phone.includes(term))
    );
  });

  // Get groups for selector - filter by selected course
  const filteredGroups = formData.courseId 
    ? groups.filter(g => g.courseId === formData.courseId)
    : groups;
  
  const availableGroups = filteredGroups.filter(g => (g.currentStudents ?? 0) < g.maxStudents);
  
  const groupOptions = filteredGroups.map(g => {
    const course = courses.find(c => c.id === g.courseId);
    const isFull = (g.currentStudents ?? 0) >= g.maxStudents;
    return {
      ...g,
      courseName: course?.name || "",
      displayName: `${g.name} - ${course?.name || 'Sin curso'}${isFull ? ' (LLENO)' : ''}`,
      isFull,
    };
  });

  // Find selected student group details
  const selectedStudentGroup = selectedStudent?.groupId 
    ? groups.find(g => g.id === selectedStudent.groupId) 
    : null;
  const selectedStudentCourse = selectedStudent?.courseId 
    ? courses.find(c => c.id === selectedStudent.courseId) 
    : null;

  const handleOpenDetails = (student: Student) => {
    setSelectedStudent(student);
    setStudentPaymentStatus(calculatePaymentStatus(student.id));
  };

  const handleCloseDetails = () => {
    setSelectedStudent(null);
    setStudentPaymentStatus(null);
  };

  const handleOpenForm = (student?: Student) => {
    if (student) {
      setEditingStudent(student);
      setFormData({
        name: student.name,
        documentNumber: student.documentNumber,
        documentType: student.documentType,
        email: student.email,
        phone: student.phone || "",
        address: student.address || "",
        birthDate: student.birthDate || "",
        guardianName: student.guardianName || "",
        guardianDocument: student.guardianDocument || "",
        guardianPhone: student.guardianPhone || "",
        courseId: student.courseId || "",
        groupId: student.groupId || "",
      });
    } else {
      setEditingStudent(null);
      setFormData({
        name: "",
        documentNumber: "",
        documentType: "cc",
        email: "",
        phone: "",
        address: "",
        birthDate: "",
        guardianName: "",
        guardianDocument: "",
        guardianPhone: "",
        courseId: "",
        groupId: "",
      });
    }
    setIsFormOpen(true);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (formData.birthDate && isMinor(formData.birthDate)) {
      if (!formData.guardianName.trim()) {
        alert("El nombre del acudiente es obligatorio para estudiantes menores de 18 años");
        return;
      }
      if (!formData.guardianPhone.trim()) {
        alert("El teléfono del acudiente es obligatorio para estudiantes menores de 18 años");
        return;
      }
    }

    if (formData.groupId) {
      const group = groups.find(g => g.id === formData.groupId);
      if (group && (group.currentStudents ?? 0) >= group.maxStudents) {
        alert("Este grupo está lleno. Selecciona otro grupo.");
        return;
      }
    }
    
    const inputData = {
      name: formData.name,
      documentNumber: formData.documentNumber,
      documentType: formData.documentType,
      email: formData.email,
      phone: formData.phone || undefined,
      address: formData.address || undefined,
      birthDate: formData.birthDate || undefined,
      guardianName: formData.guardianName || undefined,
      guardianDocument: formData.guardianDocument || undefined,
      guardianPhone: formData.guardianPhone || undefined,
      courseId: formData.courseId || undefined,
      groupId: formData.groupId || undefined,
    };

    let result;
    if (editingStudent) {
      result = await updateStudent(editingStudent.id, inputData);
    } else {
      result = await createStudent(inputData);
    }

    if (result.success) {
      setIsFormOpen(false);
      setEditingStudent(null);
    } else {
      alert(result.error || "Error al guardar");
    }
  };

  const handleDelete = async (student: Student) => {
    if (confirm(`¿Estás seguro de eliminar a ${student.name}?`)) {
      const result = await deleteStudent(student.id);
      if (!result.success) {
        alert(result.error || "Error al eliminar");
      }
    }
  };

  if (isLoading && students.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Estudiantes</h1>
        <Button onClick={() => handleOpenForm()}>Nuevo Estudiante</Button>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
          {error}
          <button onClick={refetch} className="ml-4 underline hover:no-underline">
            Reintentar
          </button>
        </div>
      )}

      {/* Search Bar */}
      <div className="mb-4">
        <Input
          placeholder="Buscar por nombre, apellido, ID, documento, email o teléfono..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="max-w-xl"
        />
      </div>

      {/* Students Table */}
      <Card>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  ID
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Nombre
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Documento
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Email
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Teléfono
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Estado
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Acciones
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {filteredStudents.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-4 py-8 text-center text-gray-500">
                    {searchTerm ? "No se encontraron estudiantes" : "No hay estudiantes registrados"}
                  </td>
                </tr>
              ) : (
                filteredStudents.map((student) => {
                  const paymentStatus = calculatePaymentStatus(student.id);
                  return (
                    <tr key={student.id} className="hover:bg-gray-50">
                      <td className="px-4 py-3 text-sm text-gray-500">
                        {student.id.substring(0, 8)}...
                      </td>
                      <td className="px-4 py-3 text-sm font-medium text-gray-900">
                        {student.name}
                      </td>
                      <td className="px-4 py-3 text-sm text-gray-500">
                        {student.documentType.toUpperCase()} {student.documentNumber}
                      </td>
                      <td className="px-4 py-3 text-sm text-gray-500">
                        {student.email || "-"}
                      </td>
                      <td className="px-4 py-3 text-sm text-gray-500">
                        {student.phone || "-"}
                      </td>
                      <td className="px-4 py-3 text-sm">
                        {paymentStatus.isPaid ? (
                          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                            Paz y salvo
                          </span>
                        ) : paymentStatus.daysOverdue > 0 ? (
                          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                            {paymentStatus.daysOverdue} días en mora
                          </span>
                        ) : (
                          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
                            Pendiente
                          </span>
                        )}
                      </td>
                      <td className="px-4 py-3 text-sm">
                        <div className="flex gap-2">
                          <Button 
                            variant="secondary" 
                            size="sm"
                            onClick={() => handleOpenDetails(student)}
                          >
                            Ver
                          </Button>
                          <Button 
                            variant="secondary" 
                            size="sm"
                            onClick={() => handleOpenForm(student)}
                          >
                            Editar
                          </Button>
                          <Button 
                            variant="danger" 
                            size="sm"
                            onClick={() => handleDelete(student)}
                          >
                            Eliminar
                          </Button>
                        </div>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Form Modal */}
      {isFormOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-y-auto p-6">
            <h2 className="text-xl font-bold mb-4">
              {editingStudent ? "Editar Estudiante" : "Nuevo Estudiante"}
            </h2>
            <form onSubmit={handleSubmit} className="space-y-4">
              <Input
                label="Nombre completo"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
              />
              
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Tipo de documento
                  </label>
                  <select
                    value={formData.documentType}
                    onChange={(e) => setFormData({ ...formData, documentType: e.target.value as typeof formData.documentType })}
                    className="w-full px-3 py-2 border border-gray-300 rounded-lg"
                    required
                  >
                    <option value="cc">Cédula de ciudadanía</option>
                    <option value="ti">Tarjeta de identidad</option>
                    <option value="ce">Cédula de extrangería</option>
                    <option value="rc">Registro civil</option>
                    <option value="nip">NIP</option>
                  </select>
                </div>
                <Input
                  label="Número de documento"
                  value={formData.documentNumber}
                  onChange={(e) => setFormData({ ...formData, documentNumber: e.target.value })}
                  required
                />
              </div>

              <Input
                label="Correo electrónico"
                type="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                required
              />

              <div className="grid grid-cols-2 gap-4">
                <Input
                  label="Teléfono"
                  type="tel"
                  value={formData.phone}
                  onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
                />
                <Input
                  label="Dirección"
                  value={formData.address}
                  onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                />
              </div>

              <Input
                label="Fecha de nacimiento"
                type="date"
                value={formData.birthDate}
                onChange={(e) => setFormData({ ...formData, birthDate: e.target.value })}
              />

              {showGuardianWarning && (
                <div className="bg-amber-50 border border-amber-200 rounded-lg p-3 text-amber-800 text-sm">
                  <strong>Atención:</strong> Este estudiante es menor de 18 años. Los campos de acudiente son obligatorios.
                </div>
              )}

              <div className="border-t pt-4">
                <h4 className="text-sm font-medium text-gray-900 mb-3">Información del acudiente</h4>
                <div className="grid grid-cols-2 gap-4">
                  <Input
                    label="Nombre del acudiente"
                    value={formData.guardianName}
                    onChange={(e) => setFormData({ ...formData, guardianName: e.target.value })}
                    required={!!(formData.birthDate && isMinor(formData.birthDate))}
                  />
                  <Input
                    label="Documento del acudiente"
                    value={formData.guardianDocument}
                    onChange={(e) => setFormData({ ...formData, guardianDocument: e.target.value })}
                    placeholder="Cédula, TI, etc."
                  />
                </div>
                <div className="grid grid-cols-2 gap-4 mt-4">
                  <Input
                    label="Teléfono del acudiente"
                    type="tel"
                    value={formData.guardianPhone}
                    onChange={(e) => setFormData({ ...formData, guardianPhone: e.target.value })}
                    required={!!(formData.birthDate && isMinor(formData.birthDate))}
                  />
                </div>
              </div>

              {/* Course Selector */}
              <SearchableSelect
                label="Curso"
                placeholder="Buscar curso..."
                value={formData.courseId}
                onChange={(id) => setFormData({ ...formData, courseId: id, groupId: "" })}
                options={courses}
                searchFields={["name", "code"] as any[]}
                displayFormatter={(course) => `${course.name} (${course.code})`}
                getItemValue={(course) => course.id}
                notFoundMessage="No se encontraron cursos"
              />

              {/* Group Selector */}
              <SearchableSelect
                label="Grupo"
                placeholder={formData.courseId ? "Buscar grupo..." : "Primero selecciona un curso"}
                value={formData.groupId}
                onChange={(id) => setFormData({ ...formData, groupId: id })}
                options={groupOptions}
                searchFields={["name", "courseName"] as any[]}
                displayFormatter={(group) => `${group.name} (${group.currentStudents}/${group.maxStudents}${group.isFull ? ' - LLENO' : ''})`}
                getItemValue={(group) => group.id}
                notFoundMessage={availableGroups.length === 0 ? "No hay grupos con cupos disponibles" : "No se encontraron grupos"}
              />
              {groupOptions.length > 0 && availableGroups.length === 0 && (
                <p className="text-amber-600 text-sm mt-1">
                  Todos los grupos de este curso están llenos
                </p>
              )}

              <div className="flex justify-end gap-3 pt-4">
                <Button type="button" variant="secondary" onClick={() => setIsFormOpen(false)}>
                  Cancelar
                </Button>
                <Button type="submit" loading={isLoading}>
                  {editingStudent ? "Actualizar" : "Crear"} estudiante
                </Button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Details Modal */}
      {selectedStudent && studentPaymentStatus && (
        <Modal
          isOpen={true}
          onClose={handleCloseDetails}
          title="Detalles del Estudiante"
        >
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <h4 className="text-sm font-medium text-gray-500">ID</h4>
                <p className="text-sm text-gray-900 font-mono">{selectedStudent.id}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-500">Nombre</h4>
                <p className="text-sm text-gray-900">{selectedStudent.name}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-500">Documento</h4>
                <p className="text-sm text-gray-900">
                  {selectedStudent.documentType.toUpperCase()} {selectedStudent.documentNumber}
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-500">Email</h4>
                <p className="text-sm text-gray-900">{selectedStudent.email || "-"}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-500">Teléfono</h4>
                <p className="text-sm text-gray-900">{selectedStudent.phone || "-"}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-500">Dirección</h4>
                <p className="text-sm text-gray-900">{selectedStudent.address || "-"}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-gray-500">Fecha de nacimiento</h4>
                <p className="text-sm text-gray-900">{formatDate(selectedStudent.birthDate)}</p>
              </div>
              {selectedStudent.birthDate && isMinor(selectedStudent.birthDate) && (
                <div>
                  <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                    Menor de edad
                  </span>
                </div>
              )}
            </div>

            {(selectedStudent.guardianName || selectedStudent.guardianDocument || selectedStudent.guardianPhone) && (
              <div className="border-t pt-4">
                <h4 className="text-sm font-medium text-gray-500 mb-2">Información del acudiente</h4>
                <div className="grid grid-cols-2 gap-4">
                  {selectedStudent.guardianName && (
                    <div>
                      <h4 className="text-xs text-gray-400">Nombre</h4>
                      <p className="text-sm text-gray-900">{selectedStudent.guardianName}</p>
                    </div>
                  )}
                  {selectedStudent.guardianDocument && (
                    <div>
                      <h4 className="text-xs text-gray-400">Documento</h4>
                      <p className="text-sm text-gray-900">{selectedStudent.guardianDocument}</p>
                    </div>
                  )}
                  {selectedStudent.guardianPhone && (
                    <div>
                      <h4 className="text-xs text-gray-400">Teléfono</h4>
                      <p className="text-sm text-gray-900">{selectedStudent.guardianPhone}</p>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Enrollment Info */}
            {(selectedStudent.courseId || selectedStudent.groupId) && (
              <div className="border-t pt-4">
                <h4 className="text-sm font-medium text-gray-500 mb-2">Inscripción</h4>
                <div className="grid grid-cols-2 gap-4">
                  {selectedStudent.courseId && (
                    <div>
                      <h4 className="text-xs text-gray-400">Curso</h4>
                      <p className="text-sm text-gray-900">{selectedStudentCourse?.name || selectedStudent.courseId}</p>
                    </div>
                  )}
                  {selectedStudent.groupId && (
                    <div>
                      <h4 className="text-xs text-gray-400">Grupo</h4>
                      <p className="text-sm text-gray-900">
                        {selectedStudentGroup?.name || selectedStudent.groupId}
                        {selectedStudentCourse && (
                          <span className="text-gray-500 ml-1">- {selectedStudentCourse.name}</span>
                        )}
                        <span className="text-gray-500 ml-1">
                          ({selectedStudentGroup?.currentStudents || 0}/{selectedStudentGroup?.maxStudents || 0} estudiantes)
                        </span>
                      </p>
                    </div>
                  )}
                </div>
              </div>
            )}

            <div className="border-t pt-4">
              <h4 className="text-sm font-medium text-gray-500 mb-3">Estado de pagos</h4>
              <div className="bg-gray-50 rounded-lg p-4 space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">Estado:</span>
                  {studentPaymentStatus.isPaid ? (
                    <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-800">
                      ✓ Paz y salvo
                    </span>
                  ) : studentPaymentStatus.daysOverdue > 0 ? (
                    <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-800">
                      ✗ {studentPaymentStatus.daysOverdue} días en mora
                    </span>
                  ) : (
                    <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-yellow-100 text-yellow-800">
                      ⏳ Pendiente
                    </span>
                  )}
                </div>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-gray-500">Último pago:</span>
                    <p className="text-gray-900">{studentPaymentStatus.lastPaymentDate}</p>
                  </div>
                  <div>
                    <span className="text-gray-500">Próximo pago:</span>
                    <p className="text-gray-900">{studentPaymentStatus.nextPaymentDate}</p>
                  </div>
                  <div>
                    <span className="text-gray-500">Total pagado:</span>
                    <p className="text-gray-900 font-medium">${studentPaymentStatus.totalPaid.toLocaleString()}</p>
                  </div>
                  {!studentPaymentStatus.isPaid && (
                    <div>
                      <span className="text-gray-500">Pendiente:</span>
                      <p className="text-gray-900 font-medium text-red-600">${studentPaymentStatus.pendingAmount.toLocaleString()}</p>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}
