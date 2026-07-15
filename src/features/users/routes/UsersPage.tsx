import { useState } from "react";
import { useUsers } from "../hooks/useUsers";
import { Card } from "../../../shared/ui/components/Card";
import { Button } from "../../../shared/ui/components/Button";
import { Input } from "../../../shared/ui/components/Input";
import { Spinner } from "../../../shared/ui/components/Spinner";
import { UserEditModal } from "../components/UserEditModal";
import type { User } from "../../../shared/types/User";

export default function UsersPage() {
  const { users, isLoading, error, createUser, updateUser, deleteUser, resetPassword, refetch } = useUsers();
  const [showForm, setShowForm] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [roleFilter, setRoleFilter] = useState<string>("all");
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    password: "",
    role: "empleado",
  });
  const [submitError, setSubmitError] = useState<string | null>(null);

  const filteredUsers = users
    .filter((user) => {
      const matchesSearch =
        user.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        user.email.toLowerCase().includes(searchTerm.toLowerCase());
      const matchesRole = roleFilter === "all" || user.role === roleFilter;
      return matchesSearch && matchesRole;
    });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitError(null);
    
    const result = await createUser(formData);
    if (result.success) {
      setShowForm(false);
      setFormData({ name: "", email: "", password: "", role: "empleado" });
    } else {
      setSubmitError(result.error || "Error al crear usuario");
    }
  };

  if (isLoading && users.length === 0) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Usuarios</h1>
        <Button onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancelar" : "Nuevo Usuario"}
        </Button>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {submitError && (
        <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
          {submitError}
        </div>
      )}

      {showForm && (
        <Card className="mb-6">
          <h2 className="text-lg font-semibold mb-4">Crear Nuevo Usuario</h2>
          <form onSubmit={handleSubmit} className="space-y-4">
            <Input
              label="Nombre"
              placeholder="Nombre completo"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
            />
            <Input
              label="Correo Electrónico"
              type="email"
              placeholder="email@ejemplo.com"
              value={formData.email}
              onChange={(e) => setFormData({ ...formData, email: e.target.value })}
              required
            />
            <Input
              label="Contraseña"
              type="password"
              placeholder="••••••••"
              value={formData.password}
              onChange={(e) => setFormData({ ...formData, password: e.target.value })}
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
            <div className="flex gap-2">
              <Button type="submit" loading={isLoading}>Crear Usuario</Button>
              <Button type="button" variant="secondary" onClick={() => setShowForm(false)}>
                Cancelar
              </Button>
            </div>
          </form>
        </Card>
      )}

      <div className="mb-4 flex gap-4">
        <div className="flex-1">
          <Input
            placeholder="Buscar usuarios..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
        <div className="w-48">
          <select
            className="w-full px-3 py-2 border border-[var(--color-foreground)]/30 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-[var(--color-background)] text-[var(--color-foreground)]"
            value={roleFilter}
            onChange={(e) => setRoleFilter(e.target.value)}
          >
            <option value="all">Todos los roles</option>
            <option value="admin">Administrador</option>
            <option value="gerente">Gerente</option>
            <option value="empleado">Empleado</option>
            <option value="profesor">Profesor</option>
          </select>
        </div>
      </div>

      {filteredUsers.length === 0 ? (
        <Card className="text-center py-12">
          <p className="text-[var(--color-foreground)]/60">No hay usuarios registrados</p>
          <Button className="mt-4" onClick={() => setShowForm(true)}>
            Crear Primer Usuario
          </Button>
        </Card>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-[var(--color-foreground)]/5">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Nombre
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Correo
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Rol
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider">
                  Acciones
                </th>
              </tr>
            </thead>
            <tbody className="bg-[var(--color-background)] divide-y divide-gray-200">
              {filteredUsers.map((user) => (
                <tr key={user.id} className="hover:bg-[var(--color-foreground)]/5">
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-[var(--color-foreground)]">
                    {user.name}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]/60">
                    {user.email}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        user.role === "admin"
                          ? "bg-purple-100 text-purple-800"
                          : user.role === "gerente"
                          ? "bg-[var(--color-primary)]/20 text-[var(--color-primary)]"
                          : user.role === "empleado"
                          ? "bg-green-100 text-green-800"
                          : "bg-[var(--color-foreground)]/10 text-[var(--color-foreground)]"
                      }`}
                    >
                      {user.role === "admin"
                        ? "Administrador"
                        : user.role === "gerente"
                        ? "Gerente"
                        : user.role === "empleado"
                        ? "Empleado"
                        : "Profesor"}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setEditingUser(user)}
                        className="text-[var(--color-primary)] hover:text-[var(--color-primary)] mr-3"
                      >
                        Editar
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          if (confirm("¿Estás seguro de eliminar este usuario?")) {
                            deleteUser(user.id);
                          }
                        }}
                        className="text-red-600 hover:text-red-900"
                        disabled={user.role === "admin"}
                      >
                        Eliminar
                      </Button>
                    </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="mt-4">
        <Button variant="secondary" onClick={refetch}>
          Actualizar
        </Button>
      </div>

      <UserEditModal
        isOpen={!!editingUser}
        onClose={() => setEditingUser(null)}
        user={editingUser}
        onSave={updateUser}
        onDelete={deleteUser}
        onResetPassword={resetPassword}
      />
    </div>
  );
}