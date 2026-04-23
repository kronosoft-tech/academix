// AccountingTable Component - Phase 8
// Reusable table component for accounting module

import { cn } from "../../../lib/utils";
import type { AccountingEntry, EmployeeListItem, PayrollEntry, PayrollRun } from "../types";

// Entry Table
interface EntriesTableProps {
  entries: AccountingEntry[];
  onRowClick?: (entry: AccountingEntry) => void;
  className?: string;
}

export function EntriesTable({ entries, onRowClick, className }: EntriesTableProps) {
  return (
    <div className={cn("w-full overflow-hidden rounded-lg border border-[var(--color-foreground)]/20", className)}>
      <table className="w-full">
        <thead className="bg-[var(--color-foreground)]/5">
          <tr>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Fecha
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Referencia
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Descripción
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Debe
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Haber
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Monto
            </th>
            <th className="h-10 px-4 text-center text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Tipo
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {entries.map((entry) => (
            <tr
              key={entry.id}
              className={cn(
                "hover:bg-[var(--color-foreground)]/5/50",
                onRowClick && "cursor-pointer"
              )}
              onClick={() => onRowClick?.(entry)}
            >
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]">
                {new Date(entry.date).toLocaleDateString("es-PE")}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm font-mono text-[var(--color-foreground)]/80">
                {entry.reference}
              </td>
              <td className="px-4 py-3 text-sm text-[var(--color-foreground)]">
                {entry.description}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/80">
                {entry.debit_account_name}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/80">
                {entry.credit_account_name}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm font-medium text-[var(--color-foreground)]">
                S/ {entry.amount.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-center">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    entry.entry_type === "manual" && "bg-[var(--color-primary)]/20 text-[var(--color-primary)]",
                    entry.entry_type === "automatic" && "bg-green-100 text-green-700",
                    entry.entry_type === "adjustment" && "bg-amber-100 text-amber-700"
                  )}
                >
                  {entry.entry_type === "manual" && "Manual"}
                  {entry.entry_type === "automatic" && "Automático"}
                  {entry.entry_type === "adjustment" && "Ajuste"}
                </span>
              </td>
            </tr>
          ))}
          {entries.length === 0 && (
            <tr>
              <td colSpan={7} className="px-4 py-8 text-center text-[var(--color-foreground)]/60">
                No hay asientos contables
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

// Employees Table
interface EmployeesTableProps {
  employees: EmployeeListItem[];
  onRowClick?: (employee: EmployeeListItem) => void;
  onEdit?: (employee: EmployeeListItem) => void;
  className?: string;
}

export function EmployeesTable({
  employees,
  onRowClick,
  onEdit,
  className,
}: EmployeesTableProps) {
  const statusColors = {
    active: "bg-green-100 text-green-700",
    inactive: "bg-[var(--color-foreground)]/10 text-[var(--color-foreground)]/80",
    suspended: "bg-amber-100 text-amber-700",
    terminated: "bg-red-100 text-red-700",
  };

  const contractLabels = {
    full_time: "Tiempo completo",
    part_time: "Medio tiempo",
    temporary: "Temporal",
    internship: "Práctica",
  };

  return (
    <div className={cn("w-full overflow-hidden rounded-lg border border-[var(--color-foreground)]/20", className)}>
      <table className="w-full">
        <thead className="bg-[var(--color-foreground)]/5">
          <tr>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              DNI
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Nombre
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Correo
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Cargo
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Departamento
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Contrato
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Salario
            </th>
            <th className="h-10 px-4 text-center text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Estado
            </th>
            {onEdit && <th className="h-10 px-4"></th>}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {employees.map((emp) => (
            <tr
              key={emp.id}
              className={cn(
                "hover:bg-[var(--color-foreground)]/5/50",
                onRowClick && "cursor-pointer"
              )}
              onClick={() => onRowClick?.(emp)}
            >
              <td className="whitespace-nowrap px-4 py-3 text-sm font-mono text-[var(--color-foreground)]/80">
                {emp.document_number}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-[var(--color-foreground)]">
                {emp.full_name}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/80">
                {emp.email}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/80">
                {emp.position}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/80">
                {emp.department}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/80">
                {contractLabels[emp.contract_type as keyof typeof contractLabels]}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm font-medium text-[var(--color-foreground)]">
                S/ {emp.base_salary.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-center">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    statusColors[emp.status as keyof typeof statusColors]
                  )}
                >
                  {emp.status === "active" && "Activo"}
                  {emp.status === "inactive" && "Inactivo"}
                  {emp.status === "suspended" && "Suspendido"}
                  {emp.status === "terminated" && "Terminado"}
                </span>
              </td>
              {onEdit && (
                <td className="whitespace-nowrap px-4 py-3 text-right">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onEdit(emp);
                    }}
                    className="rounded p-1 text-[var(--color-foreground)]/40 hover:bg-[var(--color-foreground)]/10 hover:text-[var(--color-foreground)]/80"
                  >
                    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                    </svg>
                  </button>
                </td>
              )}
            </tr>
          ))}
          {employees.length === 0 && (
            <tr>
              <td colSpan={onEdit ? 9 : 8} className="px-4 py-8 text-center text-[var(--color-foreground)]/60">
                No hay empleados
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

// Payroll Runs Table
interface PayrollRunsTableProps {
  runs: PayrollRun[];
  onRowClick?: (run: PayrollRun) => void;
  className?: string;
}

export function PayrollRunsTable({ runs, onRowClick, className }: PayrollRunsTableProps) {
  const statusColors = {
    draft: "bg-[var(--color-foreground)]/10 text-[var(--color-foreground)]/80",
    processing: "bg-[var(--color-primary)]/20 text-[var(--color-primary)]",
    completed: "bg-green-100 text-green-700",
    cancelled: "bg-red-100 text-red-700",
  };

  return (
    <div className={cn("w-full overflow-hidden rounded-lg border border-[var(--color-foreground)]/20", className)}>
      <table className="w-full">
        <thead className="bg-[var(--color-foreground)]/5">
          <tr>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Período
            </th>
            <th className="h-10 px-4 text-center text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Estado
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Empleados
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Bruto
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Deducciones
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Neto
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Creado
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {runs.map((run) => (
            <tr
              key={run.id}
              className={cn(
                "hover:bg-[var(--color-foreground)]/5/50",
                onRowClick && "cursor-pointer"
              )}
              onClick={() => onRowClick?.(run)}
            >
              <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-[var(--color-foreground)]">
                {run.period_display}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-center">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    statusColors[run.status as keyof typeof statusColors]
                  )}
                >
                  {run.status === "draft" && "Borrador"}
                  {run.status === "processing" && "Procesando"}
                  {run.status === "completed" && "Completado"}
                  {run.status === "cancelled" && "Cancelado"}
                </span>
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-[var(--color-foreground)]/80">
                {run.employee_count}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-[var(--color-foreground)]/80">
                S/ {run.total_gross.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-red-600">
                - S/ {run.total_deductions.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm font-medium text-green-700">
                S/ {run.total_net.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/60">
                {new Date(run.created_at).toLocaleDateString("es-PE")}
              </td>
            </tr>
          ))}
          {runs.length === 0 && (
            <tr>
              <td colSpan={7} className="px-4 py-8 text-center text-[var(--color-foreground)]/60">
                No hay nóminas procesadas
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

// Payroll Entries Table
interface PayrollEntriesTableProps {
  entries: PayrollEntry[];
  onRowClick?: (entry: PayrollEntry) => void;
  className?: string;
}

export function PayrollEntriesTable({ entries, onRowClick, className }: PayrollEntriesTableProps) {
  const statusColors = {
    pending: "bg-amber-100 text-amber-700",
    processed: "bg-[var(--color-primary)]/20 text-[var(--color-primary)]",
    paid: "bg-green-100 text-green-700",
    failed: "bg-red-100 text-red-700",
  };

  return (
    <div className={cn("w-full overflow-hidden rounded-lg border border-[var(--color-foreground)]/20", className)}>
      <table className="w-full">
        <thead className="bg-[var(--color-foreground)]/5">
          <tr>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Empleado
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Salario Base
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Horas Extra
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Bonificaciones
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Bruto
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              AFP
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Otros
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Deducciones
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Neto
            </th>
            <th className="h-10 px-4 text-center text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Estado
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {entries.map((entry) => (
            <tr
              key={entry.id}
              className={cn(
                "hover:bg-[var(--color-foreground)]/5/50",
                onRowClick && "cursor-pointer"
              )}
              onClick={() => onRowClick?.(entry)}
            >
              <td className="whitespace-nowrap px-4 py-3 text-sm font-medium text-[var(--color-foreground)]">
                {entry.employee_name}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-[var(--color-foreground)]/80">
                S/ {entry.base_salary.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-[var(--color-foreground)]/80">
                {entry.overtime_hours}h (+S/ {entry.overtime_amount.toFixed(2)})
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-[var(--color-foreground)]/80">
                S/ {(entry.bonuses + entry.commissions + entry.mobility + entry.food).toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm font-medium text-[var(--color-foreground)]">
                S/ {entry.gross_income.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-red-600">
                - S/ {entry.afp_deduction.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-red-600">
                - S/ {(entry.onp_deduction + entry.essalud + entry.itf + entry.other_deductions).toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm text-red-600">
                - S/ {entry.total_deductions.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm font-bold text-green-700">
                S/ {entry.net_income.toFixed(2)}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-center">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    statusColors[entry.status as keyof typeof statusColors]
                  )}
                >
                  {entry.status === "pending" && "Pendiente"}
                  {entry.status === "processed" && "Procesado"}
                  {entry.status === "paid" && "Pagado"}
                  {entry.status === "failed" && "Fallido"}
                </span>
              </td>
            </tr>
          ))}
          {entries.length === 0 && (
            <tr>
              <td colSpan={10} className="px-4 py-8 text-center text-[var(--color-foreground)]/60">
                No hay entradas de nómina
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}