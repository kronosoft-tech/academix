// DashboardCards Component - Summary cards for accounting dashboard

import { SkeletonCard } from "./SkeletonTable";

interface DashboardCardsProps {
  totalIncome: number;
  totalExpenses: number;
  netBalance: number;
  loading?: boolean;
}

function formatCurrency(amount: number): string {
  return `S/ ${amount.toLocaleString("es-PE", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

export function DashboardCards({ totalIncome, totalExpenses, netBalance, loading }: DashboardCardsProps) {
  if (loading) {
    return (
      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <SkeletonCard />
        <SkeletonCard />
        <SkeletonCard />
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
      <div className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
        <p className="text-sm text-[var(--color-foreground)]/60">Ingresos</p>
        <p className="mt-1 text-2xl font-bold text-green-600">
          {formatCurrency(totalIncome)}
        </p>
      </div>
      <div className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
        <p className="text-sm text-[var(--color-foreground)]/60">Gastos</p>
        <p className="mt-1 text-2xl font-bold text-red-600">
          {formatCurrency(totalExpenses)}
        </p>
      </div>
      <div className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
        <p className="text-sm text-[var(--color-foreground)]/60">Ganancia Neta</p>
        <p className={`mt-1 text-2xl font-bold ${netBalance >= 0 ? "text-green-600" : "text-red-600"}`}>
          {formatCurrency(netBalance)}
        </p>
      </div>
    </div>
  );
}
