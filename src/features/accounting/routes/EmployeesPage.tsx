// EmployeesPage - Phase 13
// Employee management page

import { useEffect, useState, useRef } from "react";
import { useEmployees } from "../hooks";
import { SkeletonTable } from "../components/SkeletonTable";
import { EmployeesTable } from "../components/AccountingTable";
import { animateTableRows } from "../lib/animations";
import type { CreateEmployeeRequest } from "../types";
import { cn } from "../../../lib/utils";

export default function EmployeesPage() {
  const { employees, listEmployees, createEmployee, loading, error } = useEmployees();
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState<Partial<CreateEmployeeRequest>>({});
  const [searchTerm, setSearchTerm] = useState("");
  const tableRef = useRef<HTMLTableElement>(null);

  useEffect(() => {
    listEmployees();
  }, [listEmployees]);

  // Animate on load
  useEffect(() => {
    if (!loading && employees.length > 0) {
      animateTableRows("tbody tr");
    }
  }, [loading, employees]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    listEmployees({ search: searchTerm });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await createEmployee(formData as CreateEmployeeRequest);
      setShowForm(false);
      setFormData({});
      listEmployees();
    } catch (err) {
      console.error("Failed to create employee:", err);
    }
  };

  const statusFilters = [
    { value: "", label: "Todos" },
    { value: "active", label: "Activos" },
    { value: "inactive", label: "Inactivos" },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">Empleados</h1>
          <p className="text-sm text-slate-500">Gestión de personal y nóminas</p>
        </div>
        <button
          onClick={() => setShowForm(!showForm)}
          className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
        >
          {showForm ? "Cancelar" : "Nuevo Empleado"}
        </button>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-red-700">
          {error}
        </div>
      )}

      {/* New Employee Form */}
      {showForm && (
        <form
          onSubmit={handleSubmit}
          className="rounded-lg border border-slate-200 bg-white p-6"
        >
          <h3 className="mb-4 text-lg font-semibold text-slate-900">Nuevo Empleado</h3>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Tipo Documento
              </label>
              <select
                value={formData.document_type || ""}
                onChange={(e) => setFormData({ ...formData, document_type: e.target.value as any })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              >
                <option value="">Seleccionar...</option>
                <option value="dni">DNI</option>
                <option value="ruc">RUC</option>
                <option value="ce">Carnet Extranjería</option>
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Número Documento
              </label>
              <input
                type="text"
                value={formData.document_number || ""}
                onChange={(e) => setFormData({ ...formData, document_number: e.target.value })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Nombres
              </label>
              <input
                type="text"
                value={formData.first_name || ""}
                onChange={(e) => setFormData({ ...formData, first_name: e.target.value })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Apellidos
              </label>
              <input
                type="text"
                value={formData.last_name || ""}
                onChange={(e) => setFormData({ ...formData, last_name: e.target.value })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Correo
              </label>
              <input
                type="email"
                value={formData.email || ""}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Teléfono
              </label>
              <input
                type="tel"
                value={formData.phone || ""}
                onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Cargo
              </label>
              <input
                type="text"
                value={formData.position || ""}
                onChange={(e) => setFormData({ ...formData, position: e.target.value })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Departamento
              </label>
              <input
                type="text"
                value={formData.department || ""}
                onChange={(e) => setFormData({ ...formData, department: e.target.value })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Tipo Contrato
              </label>
              <select
                value={formData.contract_type || ""}
                onChange={(e) => setFormData({ ...formData, contract_type: e.target.value as any })}
                required
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              >
                <option value="">Seleccionar...</option>
                <option value="full_time">Tiempo Completo</option>
                <option value="part_time">Medio Tiempo</option>
                <option value="temporary">Temporal</option>
                <option value="internship">Práctica</option>
              </select>
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium text-slate-700">
                Salario Base (S/)
              </label>
              <input
                type="number"
                value={formData.base_salary || ""}
                onChange={(e) => setFormData({ ...formData, base_salary: parseFloat(e.target.value) })}
                required
                step="0.01"
                min="0"
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              />
            </div>
          </div>
          <div className="mt-4 flex justify-end">
            <button
              type="submit"
              disabled={loading}
              className={cn(
                "rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700",
                loading && "cursor-not-allowed opacity-50"
              )}
            >
              {loading ? "Guardando..." : "Guardar Empleado"}
            </button>
          </div>
        </form>
      )}

      {/* Filters */}
      <div className="flex items-center gap-4 rounded-lg border border-slate-200 bg-white p-4">
        <form onSubmit={handleSearch} className="flex flex-1 items-center gap-2">
          <input
            type="text"
            placeholder="Buscar por nombre o documento..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="flex-1 rounded-md border border-slate-300 px-3 py-2 text-sm"
          />
          <button
            type="submit"
            className="rounded-md bg-slate-100 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-200"
          >
            Buscar
          </button>
        </form>
        <select
          onChange={(e) => listEmployees({ status: e.target.value })}
          className="rounded-md border border-slate-300 px-3 py-2 text-sm"
        >
          {statusFilters.map((f) => (
            <option key={f.value} value={f.value}>
              {f.label}
            </option>
          ))}
        </select>
      </div>

      {/* Employees Table */}
      {loading ? (
        <SkeletonTable rows={8} columns={7} />
      ) : (
        <div ref={tableRef}>
          <EmployeesTable employees={employees} />
        </div>
      )}
    </div>
  );
}