import { createHashRouter, Navigate } from "react-router-dom";
import { ProtectedRoute } from "./components/ProtectedRoute";

// Create router with hash routing for Tauri (no server needed)
export const router = createHashRouter([
  {
    path: "/login",
    lazy: async () => {
      const { default: LoginPage } = await import("../features/auth/routes/LoginPage");
      return { Component: LoginPage };
    },
  },
  {
    path: "/",
    element: (
      <ProtectedRoute>
        <MainLayoutWrapper />
      </ProtectedRoute>
    ),
    children: [
      {
        index: true,
        element: <Navigate to="/dashboard" replace />,
      },
      {
        path: "dashboard",
        lazy: async () => {
          const { default: DashboardPage } = await import("../features/dashboard/routes/DashboardPage");
          return { Component: DashboardPage };
        },
      },
      {
        path: "students",
        lazy: async () => {
          const { default: StudentsPage } = await import("../features/students/routes/StudentsPage");
          return { Component: StudentsPage };
        },
      },
      {
        path: "students/:id",
        lazy: async () => {
          const { default: StudentDetailPage } = await import("../features/students/routes/StudentDetailPage");
          return { Component: StudentDetailPage };
        },
      },
      {
        path: "courses",
        lazy: async () => {
          const { default: CoursesPage } = await import("../features/courses/routes/CoursesPage");
          return { Component: CoursesPage };
        },
      },
      {
        path: "courses/:id",
        lazy: async () => {
          const { default: CourseDetailPage } = await import("../features/courses/routes/CourseDetailPage");
          return { Component: CourseDetailPage };
        },
      },
      {
        path: "groups",
        lazy: async () => {
          const { default: GroupsPage } = await import("../features/groups/routes/GroupsPage");
          return { Component: GroupsPage };
        },
      },
      {
        path: "groups/:id",
        lazy: async () => {
          const { default: GroupDetailPage } = await import("../features/groups/routes/GroupDetailPage");
          return { Component: GroupDetailPage };
        },
      },
      {
        path: "payments",
        lazy: async () => {
          const { default: PaymentsPage } = await import("../features/payments/routes/PaymentsPage");
          return { Component: PaymentsPage };
        },
      },
      {
        path: "attendance",
        lazy: async () => {
          const { default: AttendancePage } = await import("../features/attendance/routes/AttendancePage");
          return { Component: AttendancePage };
        },
      },
      {
        path: "users",
        lazy: async () => {
          const { default: UsersPage } = await import("../features/users/routes/UsersPage");
          return { Component: UsersPage };
        },
      },
    ],
  },
  {
    path: "*",
    element: <Navigate to="/dashboard" replace />,
  },
]);

// Wrapper component for lazy loading MainLayout
async function MainLayoutWrapper() {
  const { default: MainLayout } = await import("./layouts/MainLayout");
  return <MainLayout />;
}
