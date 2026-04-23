import type { ReactNode } from "react";
import { cn } from "../../../lib/utils";

interface CardProps {
  children: ReactNode;
  className?: string;
  title?: string;
  description?: string;
  actions?: ReactNode;
}

export function Card({ children, className, title, description, actions }: CardProps) {
  return (
    <div className={cn("bg-[var(--color-background)] rounded-lg border border-[var(--color-foreground)]/20 shadow-sm", className)}>
      {(title || actions) && (
        <div className="px-6 py-4 border-b border-[var(--color-foreground)]/20">
          <div className="flex items-center justify-between">
            <div>
              {title && <h3 className="text-lg font-semibold text-[var(--color-foreground)]">{title}</h3>}
              {description && <p className="mt-1 text-sm text-[var(--color-foreground)]/60">{description}</p>}
            </div>
            {actions && <div className="flex gap-2">{actions}</div>}
          </div>
        </div>
      )}
      <div className="px-6 py-4">{children}</div>
    </div>
  );
}