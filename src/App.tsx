import { useEffect } from "react";
import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import { AuthProvider, useAuth } from "./shared/hooks/useAuth";
import LoginPage from "./features/auth/routes/LoginPage";
import RegisterPage from "./features/auth/routes/RegisterPage";
import MainLayout from "./app/layouts/MainLayout";
import { Spinner } from "./shared/ui/components/Spinner";
import { initTheme } from "./theme/theme";

function AppContent() {
  const { isAuthenticated, isLoading } = useAuth();

  useEffect(() => {
    initTheme();
  }, []);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[var(--color-foreground)]/5">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <Routes>
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/login" element={<LoginPage />} />
        <Route path="*" element={<Navigate to="/login" replace />} />
      </Routes>
    );
  }

  return <MainLayout />;
}

export default function App() {
  return (
    <HashRouter>
      <AuthProvider>
        <AppContent />
      </AuthProvider>
    </HashRouter>
  );
}
