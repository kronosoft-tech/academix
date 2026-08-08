import { useAuth } from "../../hooks/useAuth";

export function SubscriptionBanner() {
  const { subscription, user } = useAuth();

  if (!subscription) return null;

  const { status, daysLeft } = subscription;

  // Only show for trial or grace
  if (status !== "trial" && status !== "grace") return null;

  const userName = user?.name?.split(" ")[0] || "usuario";

  if (status === "trial" && daysLeft !== null) {
    const urgent = daysLeft <= 3;
    return (
      <div
        className={`mb-4 px-5 py-4 rounded-lg flex items-center justify-between ${urgent
            ? "bg-red-500/10 border border-red-500/20"
            : "bg-amber-500/10 border border-amber-500/20"
          }`}
      >
        <div>
          <p className={`text-sm font-medium ${urgent ? "text-red-400" : "text-amber-400"}`}>
            Hola, {userName} 👋
          </p>
          <p className={`text-sm mt-0.5 ${urgent ? "text-red-400/80" : "text-amber-400/80"}`}>
            Te quedan <strong>{daysLeft} día{daysLeft === 1 ? "" : "s"}</strong> de tu cuenta gratuita.
            Puedes adquirir tus planes en el siguiente link:
          </p>
        </div>
        <a
          href="https://academix.vercel.app/pricing"
          target="_blank"
          rel="noopener noreferrer"
          className={`shrink-0 ml-4 px-4 py-2 rounded-lg text-sm font-semibold transition-colors ${urgent
              ? "bg-red-500 text-white hover:bg-red-400"
              : "bg-amber-500 text-white hover:bg-amber-400"
            }`}
        >
          Ver planes
        </a>
      </div>
    );
  }

  if (status === "grace") {
    return (
      <div className="mb-4 px-5 py-4 rounded-lg flex items-center justify-between bg-red-500/10 border border-red-500/20">
        <div>
          <p className="text-sm font-medium text-red-400">
            Hola, {userName} ⚠️
          </p>
          <p className="text-sm mt-0.5 text-red-400/80">
            Tu pago está vencido. Actualiza tu método de pago para evitar la suspensión del servicio.
          </p>
        </div>
        <a
          href="https://academix.vercel.app/pricing"
          target="_blank"
          rel="noopener noreferrer"
          className="shrink-0 ml-4 px-4 py-2 rounded-lg text-sm font-semibold bg-red-500 text-white hover:bg-red-400 transition-colors"
        >
          Renovar
        </a>
      </div>
    );
  }

  return null;
}
