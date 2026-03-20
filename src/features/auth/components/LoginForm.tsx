import { useState } from "react";
import { useAuth } from "../../../shared/hooks/useAuth";
import { Button, Input, Card } from "../../../shared/ui";
import { validateEmailWithMessage } from "../../../shared/utils/validateEmail";
import { validatePassword } from "../../../shared/utils/validatePassword";

export function LoginForm() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [errors, setErrors] = useState<{ email?: string; password?: string }>({});
  const [isLoading, setIsLoading] = useState(false);
  const { login } = useAuth();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrors({});

    // Validate
    const emailValidation = validateEmailWithMessage(email);
    const passwordValidation = validatePassword(password);

    if (!emailValidation.valid || !passwordValidation.valid) {
      setErrors({
        email: emailValidation.message,
        password: passwordValidation.message,
      });
      return;
    }

    setIsLoading(true);
    try {
      const result = await login(email, password);
      if (!result.success) {
        setErrors({ password: result.error || "Credenciales inválidas" });
      }
    } catch {
      setErrors({ password: "Credenciales inválidas" });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <div className="text-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">Academix</h1>
        <p className="text-gray-500 mt-1">Ingresa a tu cuenta</p>
      </div>
      <form onSubmit={handleSubmit} className="space-y-4">
        <Input
          label="Correo electrónico"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          error={errors.email}
          placeholder="tu@email.com"
          autoComplete="email"
        />
        <Input
          label="Contraseña"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          error={errors.password}
          placeholder="••••••••"
          autoComplete="current-password"
        />
        <Button type="submit" className="w-full" loading={isLoading}>
          Iniciar sesión
        </Button>
      </form>
    </Card>
  );
}
