import type { InputHTMLAttributes } from "react";
import { cn } from "../../../lib/utils";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
}

export function Input({
  label,
  error,
  helperText,
  className,
  id,
  ...props
}: InputProps) {
  const inputId = id || label?.toLowerCase().replace(/\s+/g, "-");

  return (
    <div className="w-full">
      {label && (
        <label
          htmlFor={inputId}
          className="block text-sm font-medium text-[var(--color-foreground)] mb-1"
        >
          {label}
        </label>
      )}
      <input
        id={inputId}
        className={cn(
          "w-full px-3 py-2 border rounded-md text-sm transition-colors",
          "bg-[var(--color-background)] text-[var(--color-foreground)]",
          "focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-transparent",
          "disabled:opacity-50 disabled:cursor-not-allowed",
          error ? "border-red-500 focus:ring-red-500" : "border-[var(--color-foreground)]/20",
          className
        )}
        {...props}
      />
      {error && <p className="mt-1 text-sm text-red-500">{error}</p>}
      {helperText && !error && (
        <p className="mt-1 text-sm text-[var(--color-foreground)]/60">{helperText}</p>
      )}
    </div>
  );
}