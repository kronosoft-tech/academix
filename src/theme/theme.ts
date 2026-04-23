// Theme utilities for color palette customization

export const THEME_KEYS = {
  PRIMARY: "primary",
  SECONDARY: "secondary",
  TERTIARY: "tertiary",
  BACKGROUND: "background",
  FOREGROUND: "foreground",
} as const;

export const THEME_COLORS = [
  "slate",
  "gray",
  "zinc",
  "neutral",
  "stone",
  "red",
  "orange",
  "amber",
  "yellow",
  "lime",
  "green",
  "emerald",
  "teal",
  "cyan",
  "sky",
  "blue",
  "indigo",
  "violet",
  "purple",
  "fuchsia",
  "pink",
  "rose",
] as const;

export type ThemeColor = (typeof THEME_COLORS)[number];

export const DEFAULT_THEME = {
  primary: "#3b82f6",    // Azul - acciones principales, CTAs
  secondary: "#64748b",  // Gris slate - texto secundario, elementos de soporte
  tertiary: "#f59e0b",   // Naranja/Ámbar - alertas, captar atención
  background: "#ffffff", // Blanco - fondo de la app
  foreground: "#1e293b", // Gris oscuro - texto principal
} as const;

export function loadTheme(): Record<string, string> {
  if (typeof window === "undefined") return DEFAULT_THEME;
  try {
    const stored = localStorage.getItem("academix-theme");
    return stored ? JSON.parse(stored) : DEFAULT_THEME;
  } catch {
    return DEFAULT_THEME;
  }
}

export function saveTheme(theme: Record<string, string>): void {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem("academix-theme", JSON.stringify(theme));
  } catch {
    console.error("Failed to save theme to localStorage");
  }
}

export function applyTheme(theme: Record<string, string>): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  Object.entries(theme).forEach(([key, value]) => {
    root.style.setProperty(`--color-${key}`, value);
  });
}

export function initTheme(): void {
  const theme = loadTheme();
  applyTheme(theme);
}

// Map Tailwind color names to their hex shades for display
export const COLOR_SHADES: Record<string, Record<string, string>> = {
  slate: { 400: "#94a3b8", 500: "#64748b", 600: "#475569", 700: "#334155", 800: "#1e293b" },
  gray: { 400: "#9ca3af", 500: "#6b7280", 600: "#4b5563", 700: "#374151", 800: "#1f2937" },
  zinc: { 400: "#a1a1aa", 500: "#71717a", 600: "#52525b", 700: "#3f3f46", 800: "#27272a" },
  neutral: { 400: "#a3a3a3", 500: "#737373", 600: "#525252", 700: "#404040", 800: "#262626" },
  stone: { 400: "#a8a29e", 500: "#78716c", 600: "#57534e", 700: "#44403c", 800: "#292524" },
  red: { 400: "#f87171", 500: "#ef4444", 600: "#dc2626", 700: "#b91c1c", 800: "#991b1b" },
  orange: { 400: "#fb923c", 500: "#f97316", 600: "#ea580c", 700: "#c2410c", 800: "#9a3412" },
  amber: { 400: "#fbbf24", 500: "#f59e0b", 600: "#d97706", 700: "#b45309", 800: "#92400e" },
  yellow: { 400: "#facc15", 500: "#eab308", 600: "#ca8a04", 700: "#a16207", 800: "#854d0e" },
  lime: { 400: "#a3e635", 500: "#84cc16", 600: "#65a30d", 700: "#4d7c0f", 800: "#3f6212" },
  green: { 400: "#4ade80", 500: "#22c55e", 600: "#16a34a", 700: "#15803d", 800: "#166534" },
  emerald: { 400: "#34d399", 500: "#10b981", 600: "#059669", 700: "#047857", 800: "#065f46" },
  teal: { 400: "#2dd4bf", 500: "#14b8a6", 600: "#0d9488", 700: "#0f766e", 800: "#115e59" },
  cyan: { 400: "#22d3ee", 500: "#06b6d4", 600: "#0891b2", 700: "#0e7490", 800: "#155e75" },
  sky: { 400: "#38bdf8", 500: "#0ea5e9", 600: "#0284c7", 700: "#0369a1", 800: "#075985" },
  blue: { 400: "#60a5fa", 500: "#3b82f6", 600: "#2563eb", 700: "#1d4ed8", 800: "#1e40af" },
  indigo: { 400: "#818cf8", 500: "#6366f1", 600: "#4f46e5", 700: "#4338ca", 800: "#3730a3" },
  violet: { 400: "#a78bfa", 500: "#8b5cf6", 600: "#7c3aed", 700: "#6d28d9", 800: "#5b21b6" },
  purple: { 400: "#c084fc", 500: "#a855f7", 600: "#9333ea", 700: "#7e22ce", 800: "#6b21a8" },
  fuchsia: { 400: "#e879f9", 500: "#d946ef", 600: "#c026d3", 700: "#a21caf", 800: "#86198f" },
  pink: { 400: "#f472b6", 500: "#ec4899", 600: "#db2777", 700: "#be185d", 800: "#9d174d" },
  rose: { 400: "#fb7185", 500: "#f43f5e", 600: "#e11d48", 700: "#be123c", 800: "#9f1239" },
};