import { useAuth } from "./shared/hooks/useAuth";
import LoginPage from "./features/auth/routes/LoginPage";
import MainLayout from "./app/layouts/MainLayout";
import { Spinner } from "./shared/ui/components/Spinner";

export default function App() {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!isAuthenticated) {
    return <LoginPage />;
  }

  return <MainLayout />;
}
