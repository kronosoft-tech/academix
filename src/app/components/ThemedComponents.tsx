// ThemedContainer - Aplica los colores del tema automáticamente a todo su contenido
// Esto evita tener que reemplazar todos los colores en cada componente

import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

interface ThemedContainerProps {
  children: ReactNode;
  className?: string;
  variant?: "default" | "card" | "elevated" | "subtle";
}

export function ThemedContainer({ children, className, variant = "default" }: ThemedContainerProps) {
  const variants = {
    default: "bg-[var(--color-background)] text-[var(--color-foreground)]",
    card: "bg-[var(--color-background)] border border-[var(--color-foreground)]/20 rounded-lg shadow-sm",
    elevated: "bg-[var(--color-background)] border border-[var(--color-foreground)]/20 rounded-lg shadow-md",
    subtle: "bg-[color-mix(in_srgb,var(--color-secondary)_10%,var(--color-background))] rounded-lg p-4",
  };

  return (
    <div className={cn(variants[variant], className)}>
      {children}
    </div>
  );
}

// ThemedText - Para textos con el color correcto
interface ThemedTextProps {
  children: ReactNode;
  className?: string;
  variant?: "default" | "muted" | "subtle" | "inverse";
}

export function ThemedText({ children, className, variant = "default" }: ThemedTextProps) {
  const variants = {
    default: "text-[var(--color-foreground)]",
    muted: "text-[var(--color-foreground)]/60",
    subtle: "text-[var(--color-foreground)]/40",
    inverse: "text-[var(--color-background)]",
  };

  return (
    <span className={cn(variants[variant], className)}>
      {children}
    </span>
  );
}

// ThemedButton - Botón que usa los colores del tema
interface ThemedButtonProps {
  children: ReactNode;
  onClick?: () => void;
  variant?: "primary" | "secondary" | "accent" | "ghost";
  className?: string;
  disabled?: boolean;
}

export function ThemedButton({ children, onClick, variant = "primary", className, disabled }: ThemedButtonProps) {
  const variants = {
    primary: "bg-[var(--color-primary)] text-white hover:opacity-90",
    secondary: "bg-[var(--color-secondary)]/20 text-[var(--color-foreground)] hover:bg-[var(--color-secondary)]/30",
    accent: "bg-[var(--color-tertiary)] text-white hover:opacity-90",
    ghost: "bg-transparent hover:bg-[var(--color-secondary)]/10 text-[var(--color-foreground)]",
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "px-4 py-2 rounded-md font-medium transition-colors disabled:opacity-50",
        variants[variant],
        className
      )}
    >
      {children}
    </button>
  );
}

// ThemedCard - Card themed
interface ThemedCardProps {
  children: ReactNode;
  className?: string;
  title?: string;
}

export function ThemedCard({ children, className, title }: ThemedCardProps) {
  return (
    <div className={cn("bg-[var(--color-background)] border border-[var(--color-foreground)]/20 rounded-lg shadow-sm", className)}>
      {title && (
        <div className="px-6 py-4 border-b border-[var(--color-foreground)]/20">
          <h3 className="text-lg font-semibold text-[var(--color-foreground)]">{title}</h3>
        </div>
      )}
      <div className="px-6 py-4">{children}</div>
    </div>
  );
}