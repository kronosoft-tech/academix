import { useState } from "react";
import { useAuth } from "../../shared/hooks/useAuth";
import DashboardPage from "../../features/dashboard/routes/DashboardPage";
import StudentsPage from "../../features/students/routes/StudentsPage";
import CoursesPage from "../../features/courses/routes/CoursesPage";
import GroupsPage from "../../features/groups/routes/GroupsPage";
import PaymentsPage from "../../features/payments/routes/PaymentsPage";
import AttendancePage from "../../features/attendance/routes/AttendancePage";
import UsersPage from "../../features/users/routes/UsersPage";
import AccountingPage from "../../features/accounting/routes/AccountingPage";
import EmployeesPage from "../../features/accounting/routes/EmployeesPage";
import PayrollPage from "../../features/accounting/routes/PayrollPage";
import ReportsPage from "../../features/accounting/routes/ReportsPage";

type Page = "dashboard" | "students" | "courses" | "groups" | "payments" | "attendance" | "users" | "accounting" | "employees" | "payroll" | "reports";

interface NavItem {
  name: string;
  page: Page;
  allowedRoles?: Array<"admin" | "gerente" | "empleado" | "profesor">;
}

const allNavigation: NavItem[] = [
  { name: "Dashboard", page: "dashboard" },
  { name: "Estudiantes", page: "students" },
  { name: "Cursos", page: "courses" },
  { name: "Grupos", page: "groups" },
  { name: "Pagos", page: "payments", allowedRoles: ["admin", "gerente", "empleado"] },
  { name: "Asistencia", page: "attendance" },
  { name: "Contabilidad", page: "accounting", allowedRoles: ["admin", "gerente"] },
  { name: "Empleados", page: "employees", allowedRoles: ["admin", "gerente"] },
  { name: "Nómina", page: "payroll", allowedRoles: ["admin", "gerente"] },
  { name: "Reportes", page: "reports", allowedRoles: ["admin", "gerente"] },
  { name: "Usuarios", page: "users", allowedRoles: ["admin", "gerente"] },
];

// Simple icon components inline
const DashboardIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
  </svg>
);

const StudentsIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
  </svg>
);

const CoursesIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
  </svg>
);

const GroupsIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
  </svg>
);

const PaymentsIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
  </svg>
);

const AttendanceIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
  </svg>
);

const UsersIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
  </svg>
);

const AccountingIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
  </svg>
);

const EmployeesIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
  </svg>
);

const PayrollIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
  </svg>
);

const ReportsIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
  </svg>
);

const LogoutIcon = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
  </svg>
);

const icons: Record<Page, React.FC<{ className?: string }>> = {
  dashboard: DashboardIcon,
  students: StudentsIcon,
  courses: CoursesIcon,
  groups: GroupsIcon,
  payments: PaymentsIcon,
  attendance: AttendanceIcon,
  users: UsersIcon,
  accounting: AccountingIcon,
  employees: EmployeesIcon,
  payroll: PayrollIcon,
  reports: ReportsIcon,
};

export default function MainLayout() {
  const { user, logout } = useAuth();
  const [currentPage, setCurrentPage] = useState<Page>("dashboard");

  const navigation = allNavigation.filter((item) => {
    if (!item.allowedRoles) return true;
    return user && item.allowedRoles.includes(user.role);
  });

  const handleLogout = () => {
    logout();
  };

  const roleLabels: Record<string, string> = {
    admin: "Administrador",
    gerente: "Gerente",
    empleado: "Empleado",
    profesor: "Profesor",
  };

  const renderPage = () => {
    switch (currentPage) {
      case "dashboard": return <DashboardPage />;
      case "students": return <StudentsPage />;
      case "courses": return <CoursesPage />;
      case "groups": return <GroupsPage />;
      case "payments": return <PaymentsPage />;
      case "attendance": return <AttendancePage />;
      case "accounting": return <AccountingPage />;
      case "employees": return <EmployeesPage />;
      case "payroll": return <PayrollPage />;
      case "reports": return <ReportsPage />;
      case "users": return <UsersPage />;
      default: return <DashboardPage />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="flex">
        <aside className="w-64 bg-white border-r border-gray-200 fixed inset-y-0 left-0">
          <div className="h-16 flex items-center px-6 border-b border-gray-200">
            <h1 className="text-xl font-bold text-blue-600">Academix</h1>
          </div>
          <nav className="p-4 space-y-1">
            {navigation.map((item) => {
              const NavIcon = icons[item.page];
              return (
                <button
                  key={item.name}
                  onClick={() => setCurrentPage(item.page)}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                    currentPage === item.page
                      ? "bg-blue-50 text-blue-600"
                      : "text-gray-700 hover:bg-gray-100"
                  }`}
                >
                  <NavIcon className="h-5 w-5" />
                  {item.name}
                </button>
              );
            })}
          </nav>
          <div className="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-200">
            <div className="flex items-center gap-3 mb-3">
              <div className="h-8 w-8 rounded-full bg-blue-100 flex items-center justify-center">
                <span className="text-sm font-medium text-blue-600">
                  {user?.name?.charAt(0) || "U"}
                </span>
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 truncate">
                  {user?.name || "Usuario"}
                </p>
                <p className="text-xs text-gray-500 truncate capitalize">
                  {user?.role ? roleLabels[user.role] || user.role : "Usuario"}
                </p>
              </div>
            </div>
            <button
              onClick={handleLogout}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
            >
              <LogoutIcon className="h-4 w-4" />
              Cerrar sesión
            </button>
          </div>
        </aside>

        <main className="flex-1 ml-64 p-6">
          {renderPage()}
        </main>
      </div>
    </div>
  );
}
