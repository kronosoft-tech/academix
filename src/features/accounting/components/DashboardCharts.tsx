// DashboardCharts Component - Phase 10
// Chart.js visualizations for accounting dashboard

import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  LineElement,
  PointElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
} from "chart.js";
import { Bar, Doughnut, Line } from "react-chartjs-2";
import { cn } from "../../../lib/utils";

// Register Chart.js components
ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  LineElement,
  PointElement,
  ArcElement,
  Title,
  Tooltip,
  Legend
);

// Chart colors (Tailwind-inspired)
const CHART_COLORS = {
  primary: "#3b82f6",
  success: "#22c55e",
  danger: "#ef4444",
  warning: "#f59e0b",
  info: "#06b6d4",
  purple: "#8b5cf6",
  slate: "#64748b",
  slateLight: "#94a3b8",
};

// Common options for all charts
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const commonOptions: any = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      labels: {
        font: {
          family: "system-ui",
          size: 12,
        },
        color: "#64748b",
      },
    },
  },
  scales: {
    x: {
      grid: {
        display: false,
      },
      ticks: {
        font: {
          family: "system-ui",
          size: 11,
        },
        color: "#64748b",
      },
    },
    y: {
      grid: {
        color: "#e2e8f0",
      },
      ticks: {
        font: {
          family: "system-ui",
          size: 11,
        },
        color: "#64748b",
      },
    },
  },
};

// Income vs Expenses Bar Chart
interface IncomeExpensesChartProps {
  income: number;
  expenses: number;
  className?: string;
}

export function IncomeExpensesChart({
  income,
  expenses,
  className: _className,
}: IncomeExpensesChartProps) {
  void _className; // unused but kept for interface compatibility
  const data = {
    labels: ["Ingresos", "Gastos"],
    datasets: [
      {
        data: [income, expenses],
        backgroundColor: [CHART_COLORS.success, CHART_COLORS.danger],
        borderRadius: 6,
        barThickness: 48,
      },
    ],
  };

  const options = {
    ...commonOptions,
    plugins: {
      ...commonOptions.plugins,
      legend: {
        display: false,
      },
      tooltip: {
        callbacks: {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          label: (context: any) => `S/ ${context.raw.toFixed(2)}`,
        },
      },
    },
  };

  return (
    <div className="rounded-lg border border-slate-200 bg-white p-6">
      <h3 className="mb-4 text-sm font-semibold text-slate-900">Ingresos vs Gastos</h3>
      <div className="h-64">
        <Bar data={data} options={options} />
      </div>
    </div>
  );
}

// Monthly Trend Line Chart
interface MonthlyTrendChartProps {
  data: Array<{ month: string; income: number; expenses: number }>;
  className?: string;
}

export function MonthlyTrendChart({ data, className }: MonthlyTrendChartProps) {
  const chartData = {
    labels: data.map((d) => d.month),
    datasets: [
      {
        label: "Ingresos",
        data: data.map((d) => d.income),
        borderColor: CHART_COLORS.success,
        backgroundColor: "rgba(34, 197, 94, 0.1)",
        fill: true,
        tension: 0.3,
      },
      {
        label: "Gastos",
        data: data.map((d) => d.expenses),
        borderColor: CHART_COLORS.danger,
        backgroundColor: "rgba(239, 68, 68, 0.1)",
        fill: true,
        tension: 0.3,
      },
    ],
  };

  return (
    <div className={cn("rounded-lg border border-slate-200 bg-white p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-slate-900">Tendencia Mensual</h3>
      <div className="h-64">
        <Line data={chartData} options={commonOptions} />
      </div>
    </div>
  );
}

// Expense Breakdown Doughnut Chart
interface ExpenseBreakdownChartProps {
  data: Array<{ category_name: string; amount: number }>;
  className?: string;
}

export function ExpenseBreakdownChart({ data, className }: ExpenseBreakdownChartProps) {
  const colors = [
    CHART_COLORS.primary,
    CHART_COLORS.success,
    CHART_COLORS.danger,
    CHART_COLORS.warning,
    CHART_COLORS.info,
    CHART_COLORS.purple,
    CHART_COLORS.slate,
    CHART_COLORS.slateLight,
  ];

   const chartData = {
     labels: data.map((d) => d.category_name),
     datasets: [
       {
         data: data.map((d) => d.amount),
         backgroundColor: colors.slice(0, data.length),
         borderWidth: 0,
       },
     ],
   };

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: "right" as const,
        labels: {
          font: {
            family: "system-ui",
            size: 11,
          },
          color: "#64748b",
          padding: 12,
          usePointStyle: true,
        },
      },
    tooltip: {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      callbacks: {
        label: (context: any) =>
          `${context.label}: S/ ${context.raw.toFixed(2)}`,
      },
    },
    },
  };

  return (
    <div className={cn("rounded-lg border border-slate-200 bg-white p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-slate-900">Desglose de Gastos</h3>
      <div className="h-64">
        <Doughnut data={chartData} options={options} />
      </div>
    </div>
  );
}

// Income by Category Bar Chart
interface IncomeByCategoryChartProps {
  data: Array<{ category_name: string; amount: number }>;
  className?: string;
}

export function IncomeByCategoryChart({ data, className }: IncomeByCategoryChartProps) {
  const chartData = {
    labels: data.map((d) => d.category_name),
    datasets: [
      {
        label: "Ingresos",
        data: data.map((d) => d.amount),
        backgroundColor: CHART_COLORS.success,
        borderRadius: 6,
        barThickness: 32,
      },
    ],
  };

  const options = {
    ...commonOptions,
    indexAxis: "y" as const,
    plugins: {
      ...commonOptions.plugins,
      legend: {
        display: false,
      },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      tooltip: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        callbacks: {
          label: (context: any) => `S/ ${context.raw.toFixed(2)}`,
        },
      },
    },
  };

  return (
    <div className={cn("rounded-lg border border-slate-200 bg-white p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-slate-900">Ingresos por Categoría</h3>
      <div className="h-64">
        <Bar data={chartData} options={options} />
      </div>
    </div>
  );
}

// Profit Margin Gauge (using doughnut)
interface ProfitMarginChartProps {
  margin: number; // 0-100 percentage
  className?: string;
}

export function ProfitMarginChart({ margin, className }: ProfitMarginChartProps) {
  const isProfitable = margin > 0;
  const color = isProfitable ? CHART_COLORS.success : CHART_COLORS.danger;

  const data = {
    labels: ["Margen", "Restante"],
    datasets: [
      {
        data: [Math.abs(margin), 100 - Math.abs(margin)],
        backgroundColor: [color, "#e2e8f0"],
        borderWidth: 0,
      },
    ],
  };

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    cutout: "70%",
    plugins: {
      legend: {
        display: false,
      },
      tooltip: {
        enabled: false,
      },
    },
  };

  return (
    <div className={cn("rounded-lg border border-slate-200 bg-white p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-slate-900">Margen de Ganancia</h3>
      <div className="relative h-40">
        <Doughnut data={data} options={options} />
        <div className="absolute inset-0 flex items-center justify-center">
          <span
            className={cn(
              "text-2xl font-bold",
              isProfitable ? "text-green-600" : "text-red-600"
            )}
          >
            {margin.toFixed(1)}%
          </span>
        </div>
      </div>
      <p className="mt-2 text-center text-xs text-slate-500">
        {isProfitable ? "Rentable" : "No rentable"}
      </p>
    </div>
  );
}

// Payroll Summary Chart
interface PayrollSummaryChartProps {
  gross: number;
  deductions: number;
  net: number;
  className?: string;
}

export function PayrollSummaryChart({
  gross,
  deductions,
  net,
  className,
}: PayrollSummaryChartProps) {
  const data = {
    labels: ["Bruto", "Deducciones", "Neto"],
    datasets: [
      {
        data: [gross, deductions, net],
        backgroundColor: [CHART_COLORS.info, CHART_COLORS.danger, CHART_COLORS.success],
        borderRadius: 6,
        barThickness: 48,
      },
    ],
  };

  const options = {
    ...commonOptions,
    plugins: {
      ...commonOptions.plugins,
      legend: {
        display: false,
      },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      tooltip: {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        callbacks: {
          label: (context: any) => `S/ ${context.raw.toFixed(2)}`,
        },
      },
    },
  };

  return (
    <div className={cn("rounded-lg border border-slate-200 bg-white p-6", className)}>
      <h3 className="mb-4 text-sm font-semibold text-slate-900">Resumen de Nómina</h3>
      <div className="h-64">
        <Bar data={data} options={options} />
      </div>
    </div>
  );
}