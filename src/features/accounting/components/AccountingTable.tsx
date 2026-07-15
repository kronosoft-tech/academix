// AccountingTable Component - Simplified
// Table for income/expense entries

import { cn } from "../../../lib/utils";
import type { AccountingEntry } from "../types";

interface AccountingTableProps {
  entries: AccountingEntry[];
  onDelete?: (id: string) => void;
  className?: string;
}

export function AccountingTable({ entries, onDelete, className }: AccountingTableProps) {
  return (
    <div className={cn("w-full overflow-hidden rounded-lg border border-[var(--color-foreground)]/20", className)}>
      <table className="w-full">
        <thead className="bg-[var(--color-foreground)]/5">
          <tr>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Fecha
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Tipo
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Categoría
            </th>
            <th className="h-10 px-4 text-left text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Descripción
            </th>
            <th className="h-10 px-4 text-right text-xs font-medium uppercase tracking-wide text-[var(--color-foreground)]/60">
              Monto
            </th>
            {onDelete && <th className="h-10 px-4 w-12"></th>}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {entries.map((entry) => (
            <tr key={entry.id} className="hover:bg-[var(--color-foreground)]/5">
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]">
                {new Date(entry.date).toLocaleDateString("es-PE")}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-center">
                <span
                  className={cn(
                    "inline-flex rounded-full px-2 py-0.5 text-xs font-medium",
                    entry.entry_type === "income"
                      ? "bg-green-100 text-green-700"
                      : "bg-red-100 text-red-700"
                  )}
                >
                  {entry.entry_type === "income" ? "Ingreso" : "Gasto"}
                </span>
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-sm text-[var(--color-foreground)]/80">
                {entry.category}
              </td>
              <td className="px-4 py-3 text-sm text-[var(--color-foreground)]">
                {entry.description}
              </td>
              <td className="whitespace-nowrap px-4 py-3 text-right text-sm font-medium text-[var(--color-foreground)]">
                S/ {entry.amount.toFixed(2)}
              </td>
              {onDelete && (
                <td className="whitespace-nowrap px-4 py-3 text-center">
                  <button
                    onClick={() => onDelete(entry.id)}
                    className="rounded p-1 text-red-400 hover:bg-red-50 hover:text-red-600"
                    title="Eliminar"
                  >
                    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </td>
              )}
            </tr>
          ))}
          {entries.length === 0 && (
            <tr>
              <td colSpan={onDelete ? 6 : 5} className="px-4 py-8 text-center text-[var(--color-foreground)]/60">
                No hay asientos en este período
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
