// DashboardCharts Component - Using Recharts
// Charts for accounting dashboard

import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
} from "recharts";
import { cn } from "../../../lib/utils";

const CHART_COLORS = ["#3b82f6", "#22c55e", "#ef4444", "#f59e0b", "#06b6d4", "#8b5cf6", "#64748b", "#94a3b8"];

interface IncomeExpensesChartProps {
  income: number;
  expenses: number;
  className?: string;
}

export function IncomeExpensesChart({ income, expenses, className }: IncomeExpensesChartProps) {
  const data = [
    { name: "Ingresos", value: income },
    { name: "Gastos", value: expenses },
  ];

  return (
    <div className={cn("rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-[var(--color-foreground)]">Ingresos vs Gastos</h3>
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
            <XAxis dataKey="name" tick={{ fill: "#64748b", fontSize: 12 }} />
            <YAxis tick={{ fill: "#64748b", fontSize: 12 }} />
            <Tooltip formatter={(value) => [`S/ ${Number(value).toFixed(2)}`, ""]} />
            <Bar dataKey="value" radius={[6, 6, 0, 0]} barSize={48}>
              <Cell fill="#22c55e" />
              <Cell fill="#ef4444" />
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

interface MonthlyTrendChartProps {
  data: Array<{ month: string; income: number; expenses: number }>;
  className?: string;
}

export function MonthlyTrendChart({ data, className }: MonthlyTrendChartProps) {
  return (
    <div className={cn("rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-[var(--color-foreground)]">Tendencia Mensual</h3>
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data}>
            <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
            <XAxis dataKey="month" tick={{ fill: "#64748b", fontSize: 12 }} />
            <YAxis tick={{ fill: "#64748b", fontSize: 12 }} />
            <Tooltip formatter={(value) => [`S/ ${Number(value).toFixed(2)}`, ""]} />
            <Legend />
            <Line type="monotone" dataKey="income" stroke="#22c55e" strokeWidth={2} name="Ingresos" />
            <Line type="monotone" dataKey="expenses" stroke="#ef4444" strokeWidth={2} name="Gastos" />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

interface ExpenseBreakdownChartProps {
  data: Array<{ category_name: string; amount: number }>;
  className?: string;
}

export function ExpenseBreakdownChart({ data, className }: ExpenseBreakdownChartProps) {
  const renderLabel = (props: { category_name?: string; percent?: number }) => {
    const { category_name, percent } = props;
    if (!category_name || percent === undefined) return "";
    return `${category_name} ${(percent * 100).toFixed(0)}%`;
  };

  return (
    <div className={cn("rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-[var(--color-foreground)]">Desglose de Gastos</h3>
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={data}
              dataKey="amount"
              nameKey="category_name"
              cx="50%"
              cy="50%"
              outerRadius={80}
              label={renderLabel}
            >
              {data.map((_, index) => (
                <Cell key={`cell-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
              ))}
            </Pie>
            <Tooltip formatter={(value) => [`S/ ${Number(value).toFixed(2)}`, ""]} />
            <Legend />
          </PieChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

interface IncomeBreakdownChartProps {
  data: Array<{ category_name: string; amount: number }>;
  className?: string;
}

export function IncomeBreakdownChart({ data, className }: IncomeBreakdownChartProps) {
  const renderLabel = (props: { category_name?: string; percent?: number }) => {
    const { category_name, percent } = props;
    if (!category_name || percent === undefined) return "";
    return `${category_name} ${(percent * 100).toFixed(0)}%`;
  };

  return (
    <div className={cn("rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-[var(--color-foreground)]">Desglose de Ingresos</h3>
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={data}
              dataKey="amount"
              nameKey="category_name"
              cx="50%"
              cy="50%"
              outerRadius={80}
              label={renderLabel}
            >
              {data.map((_, index) => (
                <Cell key={`cell-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
              ))}
            </Pie>
            <Tooltip formatter={(value) => [`S/ ${Number(value).toFixed(2)}`, ""]} />
            <Legend />
          </PieChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
