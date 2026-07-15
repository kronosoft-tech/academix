// Profile form for admin self-edition with name, email, and password change

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAuth } from "../../shared/hooks/useAuth";

interface ProfileFormData {
  name: string;
  email: string;
}

interface PasswordFormData {
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
}

interface UpdateProfileResponse {
  success: boolean;
  user?: {
    id: string;
    email: string;
    name: string;
    role: string;
  };
  error?: string;
}

interface ChangePasswordResponse {
  success: boolean;
  error?: string;
}

export function ProfileForm() {
  const { user, token } = useAuth();
  const [profileData, setProfileData] = useState<ProfileFormData>({
    name: "",
    email: "",
  });
  const [passwordData, setPasswordData] = useState<PasswordFormData>({
    currentPassword: "",
    newPassword: "",
    confirmPassword: "",
  });
  const [profileLoading, setProfileLoading] = useState(false);
  const [passwordLoading, setPasswordLoading] = useState(false);
  const [profileMessage, setProfileMessage] = useState<{ type: "success" | "error"; text: string } | null>(
    null
  );
  const [passwordMessage, setPasswordMessage] = useState<{ type: "success" | "error"; text: string } | null>(
    null
  );
  const [showPasswordSection, setShowPasswordSection] = useState(false);

  useEffect(() => {
    if (user) {
      setProfileData({
        name: user.name || "",
        email: user.email || "",
      });
    }
  }, [user]);

  const handleProfileSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setProfileLoading(true);
    setProfileMessage(null);

    try {
      const response = await invoke<UpdateProfileResponse>("update_profile", {
        request: {
          token,
          name: profileData.name,
          email: profileData.email,
        },
      });

      if (response.success && response.user) {
        setProfileMessage({ type: "success", text: "Perfil actualizado exitosamente" });
      } else {
        setProfileMessage({ type: "error", text: response.error || "Error al actualizar perfil" });
      }
    } catch (error) {
      setProfileMessage({
        type: "error",
        text: error instanceof Error ? error.message : "Error al actualizar perfil",
      });
    } finally {
      setProfileLoading(false);
    }
  };

  const handlePasswordSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setPasswordMessage(null);

    if (passwordData.newPassword !== passwordData.confirmPassword) {
      setPasswordMessage({ type: "error", text: "Las contraseñas no coinciden" });
      return;
    }

    if (passwordData.newPassword.length < 6) {
      setPasswordMessage({ type: "error", text: "La nueva contraseña debe tener al menos 6 caracteres" });
      return;
    }

    setPasswordLoading(true);

    try {
      const response = await invoke<ChangePasswordResponse>("change_password", {
        request: {
          token,
          current_password: passwordData.currentPassword,
          new_password: passwordData.newPassword,
        },
      });

      if (response.success) {
        setPasswordMessage({ type: "success", text: "Contraseña cambiada exitosamente" });
        setPasswordData({ currentPassword: "", newPassword: "", confirmPassword: "" });
        setShowPasswordSection(false);
      } else {
        setPasswordMessage({ type: "error", text: response.error || "Error al cambiar contraseña" });
      }
    } catch (error) {
      setPasswordMessage({
        type: "error",
        text: error instanceof Error ? error.message : "Error al cambiar contraseña",
      });
    } finally {
      setPasswordLoading(false);
    }
  };

  return (
    <div className="space-y-8">
      {/* Profile Section */}
      <div className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
        <h2 className="text-lg font-semibold text-[var(--color-foreground)] mb-4">Información Personal</h2>

        <form onSubmit={handleProfileSubmit} className="space-y-4">
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)]/80 mb-1">
                Nombre
              </label>
              <input
                type="text"
                value={profileData.name}
                onChange={(e) => setProfileData({ ...profileData, name: e.target.value })}
                className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm bg-[var(--color-background)] text-[var(--color-foreground)]"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)]/80 mb-1">
                Correo electrónico
              </label>
              <input
                type="email"
                value={profileData.email}
                onChange={(e) => setProfileData({ ...profileData, email: e.target.value })}
                className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm bg-[var(--color-background)] text-[var(--color-foreground)]"
                required
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-[var(--color-foreground)]/80 mb-1">
              Rol
            </label>
            <input
              type="text"
              value={user?.role || ""}
              className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm bg-[var(--color-background)]/50 text-[var(--color-foreground)]"
              disabled
            />
            <p className="mt-1 text-xs text-[var(--color-foreground)]/60">
              El rol no se puede cambiar
            </p>
          </div>

          {profileMessage && (
            <div
              className={`rounded-md px-4 py-3 text-sm ${
                profileMessage.type === "success"
                  ? "bg-green-500/10 text-green-600"
                  : "bg-red-500/10 text-red-600"
              }`}
            >
              {profileMessage.text}
            </div>
          )}

          <div className="flex justify-end">
            <button
              type="submit"
              disabled={profileLoading}
              className="px-4 py-2 bg-[var(--color-primary)] text-white rounded-md text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
            >
              {profileLoading ? "Guardando..." : "Guardar Cambios"}
            </button>
          </div>
        </form>
      </div>

      {/* Password Section */}
      <div className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-[var(--color-foreground)]">Cambiar Contraseña</h2>
          {!showPasswordSection && (
            <button
              onClick={() => setShowPasswordSection(true)}
              className="text-sm text-[var(--color-primary)] hover:underline"
            >
              Cambiar contraseña
            </button>
          )}
        </div>

        {showPasswordSection ? (
          <form onSubmit={handlePasswordSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-[var(--color-foreground)]/80 mb-1">
                Contraseña actual
              </label>
              <input
                type="password"
                value={passwordData.currentPassword}
                onChange={(e) => setPasswordData({ ...passwordData, currentPassword: e.target.value })}
                className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm bg-[var(--color-background)] text-[var(--color-foreground)]"
                required
              />
            </div>

            <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div>
                <label className="block text-sm font-medium text-[var(--color-foreground)]/80 mb-1">
                  Nueva contraseña
                </label>
                <input
                  type="password"
                  value={passwordData.newPassword}
                  onChange={(e) => setPasswordData({ ...passwordData, newPassword: e.target.value })}
                  className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm bg-[var(--color-background)] text-[var(--color-foreground)]"
                  required
                  minLength={6}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-[var(--color-foreground)]/80 mb-1">
                  Confirmar nueva contraseña
                </label>
                <input
                  type="password"
                  value={passwordData.confirmPassword}
                  onChange={(e) => setPasswordData({ ...passwordData, confirmPassword: e.target.value })}
                  className="w-full rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm bg-[var(--color-background)] text-[var(--color-foreground)]"
                  required
                  minLength={6}
                />
              </div>
            </div>

            {passwordMessage && (
              <div
                className={`rounded-md px-4 py-3 text-sm ${
                  passwordMessage.type === "success"
                    ? "bg-green-500/10 text-green-600"
                    : "bg-red-500/10 text-red-600"
                }`}
              >
                {passwordMessage.text}
              </div>
            )}

            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={() => {
                  setShowPasswordSection(false);
                  setPasswordData({ currentPassword: "", newPassword: "", confirmPassword: "" });
                  setPasswordMessage(null);
                }}
                className="px-4 py-2 border border-[var(--color-foreground)]/30 rounded-md text-sm font-medium hover:bg-[var(--color-foreground)]/5 transition-colors"
              >
                Cancelar
              </button>
              <button
                type="submit"
                disabled={passwordLoading}
                className="px-4 py-2 bg-[var(--color-primary)] text-white rounded-md text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
              >
                {passwordLoading ? "Cambiando..." : "Cambiar Contraseña"}
              </button>
            </div>
          </form>
        ) : (
          <p className="text-sm text-[var(--color-foreground)]/60">
            Cambia tu contraseña para mantener tu cuenta segura
          </p>
        )}
      </div>
    </div>
  );
}