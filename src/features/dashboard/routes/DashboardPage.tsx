import { useDashboard } from "../hooks/useDashboard";
import { Card } from "../../../shared/ui/components/Card";
import { Spinner } from "../../../shared/ui/components/Spinner";

export default function DashboardPage() {
  const { stats, isLoading, error } = useDashboard();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold text-[var(--color-foreground)] mb-6">Dashboard</h1>

      <div 
        className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8"
      >
        <StatCard
          title="Estudiantes"
          value={stats?.totalStudents ?? 0}
          icon={
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
            </svg>
          }
          color="blue"
        />
        <StatCard
          title="Cursos"
          value={stats?.totalCourses ?? 0}
          icon={
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
          }
          color="green"
        />
        <StatCard
          title="Grupos"
          value={stats?.totalGroups ?? 0}
          icon={
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
          }
          color="purple"
        />
        <StatCard
          title="Pagos Pendientes"
          value={stats?.pendingPayments ?? 0}
          icon={
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          }
          color={stats?.pendingPayments ? "orange" : "green"}
        />
      </div>

      {/* Quick Actions */}
      <h2 className="text-lg font-semibold text-[var(--color-foreground)] mb-4">Acciones Rápidas</h2>
      <div 
        className="grid grid-cols-1 md:grid-cols-3 gap-4"
      >
        <QuickActionCard
          title="Registrar Estudiante"
          description="Agregar un nuevo estudiante al sistema"
          href="/students"
          icon={
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" />
            </svg>
          }
        />
        <QuickActionCard
          title="Crear Grupo"
          description="Registrar un nuevo grupo de curso"
          href="/groups"
          icon={
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
          }
        />
        <QuickActionCard
          title="Registrar Pago"
          description="Procesar un nuevo pago"
          href="/payments"
          icon={
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
          }
        />
      </div>
    </div>
  );
}

interface StatCardProps {
  title: string;
  value: number;
  icon: React.ReactNode;
  color: "blue" | "green" | "purple" | "orange";
}

function StatCard({ title, value, icon, color }: StatCardProps) {
  const colorStyles = {
    blue: "bg-[var(--color-primary)]/10 text-[var(--color-tertiary)]",
    green: "bg-[var(--color-secondary)]/10 text-[var(--color-tertiary)]",
    purple: "bg-[var(--color-tertiary)]/10 text-[var(--color-tertiary)]",
    orange: "bg-[var(--color-tertiary)]/10 text-[var(--color-tertiary)]",
  };

  return (
    <Card className="flex items-center gap-4">
      <div className={`p-3 rounded-lg ${colorStyles[color]}`}>{icon}</div>
      <div>
        <p className="text-sm text-[var(--color-foreground)]/60">{title}</p>
        <p className="text-2xl font-bold text-[var(--color-foreground)]">{value}</p>
      </div>
    </Card>
  );
}

interface QuickActionCardProps {
  title: string;
  description: string;
  href: string;
  icon: React.ReactNode;
}

function QuickActionCard({ title, description, href, icon }: QuickActionCardProps) {
  return (
    <a
      href={href}
      className="block p-4 bg-[var(--color-background)] border border-[var(--color-foreground)]/20 rounded-lg hover:border-blue-300 hover:shadow-md transition-all"
    >
      <div className="flex items-center gap-3">
        <div className="p-2 bg-[var(--color-primary)]/10 text-[var(--color-primary)] rounded-lg">{icon}</div>
        <div>
          <p className="font-medium text-[var(--color-foreground)]">{title}</p>
          <p className="text-sm text-[var(--color-foreground)]/60">{description}</p>
        </div>
      </div>
    </a>
  );
}