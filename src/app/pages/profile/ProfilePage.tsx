// Profile page for admin self-edition

import { ProfileForm } from "../../components/ProfileForm";

export default function ProfilePage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-[var(--color-foreground)]">Mi Perfil</h1>
        <p className="mt-1 text-sm text-[var(--color-foreground)]/60">
          Actualiza tu información personal y contraseña
        </p>
      </div>

      <div className="max-w-2xl">
        <ProfileForm />
      </div>
    </div>
  );
}