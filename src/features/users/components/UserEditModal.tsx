import { useState, useEffect } from "react";
import { Modal } from "../../../shared/ui/components/Modal";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";
import type { User } from "../../../shared/types/User";

interface UserEditModalProps {
  isOpen: boolean;
  onClose: () => void;
  user: User | null;
  onSave: (id: string, data: {
    name?: string;
    email?: string;
    role?: string;
    password?: string;
  }) => Promise<{ success: boolean; error?: string }>;
  onDelete?: (id: string) => Promise<{ success: boolean; error?: string }>;
  onResetPassword?: (id: string, newPassword: string) => Promise<{ success: boolean; error?: string }>;
}

export function UserEditModal({
  isOpen,
  onClose,
  user,
  onSave,
  onDelete,
  onResetPassword,
}: UserEditModalProps) {
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    role: "empleado" as string,
    newPassword: "",
    confirmPassword: "",
  });
  const [isSaving, setIsSaving] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [showPasswordChange, setShowPasswordChange] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  useEffect(() => {
    if (user) {
      setFormData({
        name: user.name,
        email: user.email,
        role: user.role,
        newPassword: "",
        confirmPassword: "",
      });
      setShowPasswordChange(false);
      setDeleteConfirm(false);
      setError(null);
      setSuccessMessage(null);
    }
  }, [user, isOpen]);

  const handleSave = async () => {
    setError(null);
    setIsSaving(true);

    const updateData: {
      name?: string;
      email?: string;
      role?: string;
      password?: string;
    } = {
      name: formData.name,
      email: formData.email,
      role: formData.role,
    };

    // Only include password if user provided one
    if (showPasswordChange && formData.newPassword) {
      if (formData.newPassword.length < 8) {
        setError("La contraseña debe tener al menos 8 caracteres");
        setIsSaving(false);
        return;
      }
      if (formData.newPassword !== formData.confirmPassword) {
        setError("Las contraseñas no coinciden");
        setIsSaving(false);
        return;
      }
      updateData.password = formData.newPassword;
    }

    const result = await onSave(user!.id, updateData);
    
    if (result.success) {
      setSuccessMessage("Usuario actualizado correctamente");
      setTimeout(() => {
        onClose();
        setSuccessMessage(null);
      }, 1500);
    } else {
      setError(result.error || "Error al actualizar usuario");
    }
    setIsSaving(false);
  };

  const handleDelete = async () => {
    if (!deleteConfirm) {
      setDeleteConfirm(true);
      return;
    }

    setError(null);
    setIsDeleting(true);

    const result = await onDelete!(user!.id);
    
    if (result.success) {
      onClose();
    } else {
      setError(result.error || "Error al eliminar usuario");
    }
    setIsDeleting(false);
  };

  const handleResetPassword = async () => {
    setError(null);
    setIsResetting(true);

    // Generate a random password or use default
    const defaultPassword = "Academix2024!";
    const result = await onResetPassword!(user!.id, defaultPassword);
    
    if (result.success) {
      setSuccessMessage(`Contraseña reseteada a: ${defaultPassword}`);
      setFormData(prev => ({ ...prev, newPassword: "", confirmPassword: "" }));
      setShowPasswordChange(false);
    } else {
      setError(result.error || "Error al resetear contraseña");
    }
    setIsResetting(false);
  };

  const isAdminUser = user?.role === "admin";

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Editar Usuario" size="md">
      {successMessage && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 text-green-800 rounded-lg">
          {successMessage}
        </div>
      )}

      {error && (
        <div className="mb-4 p-3 bg-red-50 border border-red-200 text-red-800 rounded-lg">
          {error}
        </div>
      )}

      <div className="space-y-4">
        <Input
          label="Nombre"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          required
        />

        <Input
          label="Correo Electrónico"
          type="email"
          value={formData.email}
          onChange={(e) => setFormData({ ...formData, email: e.target.value })}
          required
        />

        <div>
          <label className="block text-sm font-medium text-[var(--color-foreground)] mb-1">
            Rol
          </label>
          <select
            className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            value={formData.role}
            onChange={(e) => setFormData({ ...formData, role: e.target.value })}
          >
            <option value="admin">Administrador</option>
            <option value="gerente">Gerente</option>
            <option value="empleado">Empleado</option>
            <option value="profesor">Profesor</option>
          </select>
        </div>

        {/* Password Change Section */}
        <div className="border-t pt-4 mt-4">
          {!showPasswordChange ? (
            <div className="flex gap-2">
              <Button
                type="button"
                variant="secondary"
                onClick={() => setShowPasswordChange(true)}
              >
                Cambiar Contraseña
              </Button>
              {onResetPassword && !isAdminUser && (
                <Button
                  type="button"
                  variant="secondary"
                  onClick={handleResetPassword}
                  loading={isResetting}
                >
                  Resetear Contraseña
                </Button>
              )}
            </div>
          ) : (
            <div className="space-y-3">
              <Input
                label="Nueva Contraseña"
                type="password"
                placeholder="Mínimo 8 caracteres"
                value={formData.newPassword}
                onChange={(e) => setFormData({ ...formData, newPassword: e.target.value })}
              />
              <Input
                label="Confirmar Contraseña"
                type="password"
                placeholder="Repite la contraseña"
                value={formData.confirmPassword}
                onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
              />
              <Button
                type="button"
                variant="secondary"
                onClick={() => {
                  setShowPasswordChange(false);
                  setFormData(prev => ({ ...prev, newPassword: "", confirmPassword: "" }));
                }}
              >
                Cancelar
              </Button>
            </div>
          )}
        </div>

        {/* Action Buttons */}
        <div className="flex justify-between pt-4 border-t">
          <div>
            {onDelete && !isAdminUser && (
              <Button
                type="button"
                variant="danger"
                onClick={handleDelete}
                loading={isDeleting}
              >
                {deleteConfirm ? "Confirmar Eliminación" : "Eliminar Usuario"}
              </Button>
            )}
            {isAdminUser && (
              <p className="text-sm text-[var(--color-foreground)]/60">
                Los usuarios administradores no pueden ser eliminados
              </p>
            )}
          </div>
          <div className="flex gap-2">
            <Button type="button" variant="secondary" onClick={onClose}>
              Cancelar
            </Button>
            <Button onClick={handleSave} loading={isSaving}>
              Guardar Cambios
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  );
}