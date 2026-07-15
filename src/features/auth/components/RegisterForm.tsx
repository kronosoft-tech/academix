import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { Button, Input, Card } from "../../../shared/ui";
import { validateEmailWithMessage } from "../../../shared/utils/validateEmail";
import { validatePassword } from "../../../shared/utils/validatePassword";

interface RegisterFormData {
  name: string;
  email: string;
  password: string;
  confirmPassword: string;
}

interface FormErrors {
  name?: string;
  email?: string;
  password?: string;
  confirmPassword?: string;
}

export function RegisterForm() {
  const [formData, setFormData] = useState<RegisterFormData>({
    name: "",
    email: "",
    password: "",
    confirmPassword: "",
  });
  const [errors, setErrors] = useState<FormErrors>({});
  const [isLoading, setIsLoading] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const navigate = useNavigate();

  const handleChange = (field: keyof RegisterFormData) => (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData((prev) => ({ ...prev, [field]: e.target.value }));
    // Clear error when user starts typing
    if (errors[field]) {
      setErrors((prev) => ({ ...prev, [field]: undefined }));
    }
  };

  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    // Name validation
    if (!formData.name.trim()) {
      newErrors.name = "El nombre es requerido";
    } else if (formData.name.trim().length < 2) {
      newErrors.name = "El nombre debe tener al menos 2 caracteres";
    }

    // Email validation
    const emailValidation = validateEmailWithMessage(formData.email);
    if (!emailValidation.valid) {
      newErrors.email = emailValidation.message;
    }

    // Password validation
    const passwordValidation = validatePassword(formData.password);
    if (!passwordValidation.valid) {
      newErrors.password = passwordValidation.message;
    }

    // Confirm password validation
    if (!formData.confirmPassword) {
      newErrors.confirmPassword = "Confirma tu contraseña";
    } else if (formData.password !== formData.confirmPassword) {
      newErrors.confirmPassword = "Las contraseñas no coinciden";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSuccessMessage(null);

    if (!validateForm()) {
      return;
    }

    setIsLoading(true);
    try {
      const response = await invoke<{
        success: boolean;
        user?: {
          id: string;
          email: string;
          name: string;
        };
        error?: string;
      }>("register_user", {
        request: {
          name: formData.name.trim(),
          email: formData.email.trim().toLowerCase(),
          password: formData.password,
        },
      });

      if (response.success) {
        setSuccessMessage("¡Cuenta creada exitosamente! Redirigiendo al login...");
        // Clear form
        setFormData({ name: "", email: "", password: "", confirmPassword: "" });
        // Redirect to login after 2 seconds
        setTimeout(() => {
          navigate("/login");
        }, 2000);
      } else {
        // Handle backend errors
        if (response.error?.toLowerCase().includes("email") || response.error?.toLowerCase().includes("duplicate")) {
          setErrors({ email: "Este correo electrónico ya está registrado" });
        } else {
          setErrors({ email: response.error || "Error al crear la cuenta" });
        }
      }
    } catch (error) {
      setErrors({ email: error instanceof Error ? error.message : "Error al crear la cuenta" });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <div className="text-center mb-6">
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Academix</h1>
        <p className="text-[var(--color-foreground)]/60 mt-1">Crea tu cuenta</p>
      </div>

      {successMessage && (
        <div className="mb-4 p-3 bg-green-100 border border-green-300 rounded-md text-green-800 text-sm text-center">
          {successMessage}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-4">
        <Input
          label="Nombre completo"
          type="text"
          value={formData.name}
          onChange={handleChange("name")}
          error={errors.name}
          placeholder="Tu nombre"
          autoComplete="name"
        />
        <Input
          label="Correo electrónico"
          type="email"
          value={formData.email}
          onChange={handleChange("email")}
          error={errors.email}
          placeholder="tu@email.com"
          autoComplete="email"
        />
        <Input
          label="Contraseña"
          type="password"
          value={formData.password}
          onChange={handleChange("password")}
          error={errors.password}
          placeholder="Mínimo 6 caracteres"
          autoComplete="new-password"
        />
        <Input
          label="Confirmar contraseña"
          type="password"
          value={formData.confirmPassword}
          onChange={handleChange("confirmPassword")}
          error={errors.confirmPassword}
          placeholder="Repite tu contraseña"
          autoComplete="new-password"
        />
        <Button type="submit" className="w-full" loading={isLoading}>
          Crear cuenta
        </Button>
      </form>

      <div className="mt-4 text-center text-sm">
        <span className="text-[var(--color-foreground)]/60">¿Ya tienes cuenta? </span>
        <button
          type="button"
          onClick={() => navigate("/login")}
          className="text-[var(--color-primary)] hover:underline font-medium"
        >
          Inicia sesión
        </button>
      </div>
    </Card>
  );
}