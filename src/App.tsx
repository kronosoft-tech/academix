import { useEffect } from "react";
import { useAuth } from "./shared/hooks/useAuth";
import LoginPage from "./features/auth/routes/LoginPage";
import MainLayout from "./app/layouts/MainLayout";
import { Spinner } from "./shared/ui/components/Spinner";
import { initTheme } from "./theme/theme";

export default function App() {
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
    return <LoginPage />;
  }

  return <MainLayout />;
}
