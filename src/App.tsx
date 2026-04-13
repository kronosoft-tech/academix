import { useAuth } from "./shared/hooks/useAuth";
import LoginPage from "./features/auth/routes/LoginPage";
import MainLayout from "./app/layouts/MainLayout";
import { Spinner } from "./shared/ui/components/Spinner";

console.log("App.tsx: Loading...");

export default function App() {
  console.log("App.tsx: Rendering");
  const { isAuthenticated, isLoading } = useAuth();
  console.log("App.tsx: auth state - loading:", isLoading, "authenticated:", isAuthenticated);

  if (isLoading) {
    console.log("App.tsx: Showing spinner");
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!isAuthenticated) {
    console.log("App.tsx: Showing login");
    return <LoginPage />;
  }

  console.log("App.tsx: Showing MainLayout");
  return <MainLayout />;
}
