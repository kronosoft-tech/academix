# Technical Design: Color Palette Customization

## Project: Academix
## Change: color-pallete-customization

---

## Technical Approach

For Tailwind CSS v4 with runtime customization:

1. Use CSS custom properties with @property for reactivity
2. Update via JavaScript when user changes colors
3. Store in localStorage with key "academix-theme"
4. Apply to :root on page load

### Tailwind CSS v4 @property Setup

- Add @property in index.css to enable reactivity
- Use CSS variables in @theme that map to semantic names
- Update via document.documentElement.style.setProperty()

---

## Architecture Decisions

### 1. CSS Variables vs Tailwind Classes

- **Choice**: CSS custom properties in :root
- **Rationale**: Tailwind v4 supports CSS-first config, easy to update at runtime

### 2. Color Picker UI

- **Choice**: Grid of Tailwind color squares per category
- **Rationale**: Simple, no external dependencies, fits existing UI

### 3. Dropdown Implementation

- **Choice**: Native HTML select or inline dropdown
- **Rationale**: Keep simple, maintain SPA state structure in MainLayout

### 4. Persistence

- **Choice**: localStorage with key "academix-theme"
- **Rationale**: Simple, no backend needed, desktop app context

---

## Data Flow

```
User selects color → updateCSSVariable(name, value) → localStorage.setItem("academix-theme", JSON.stringify(theme))
                                              ↓
                                    document.documentElement.style.setProperty(--color-primary, value)
                                              ↓
                                    loadColors() → localStorage.getItem("academix-theme") on app init
```

---

## File Changes

| File | Action | Description |
|------|--------|-------------|
| src/app/pages/settings/SettingsPage.tsx | Create | New settings page with color pickers |
| src/theme/theme.ts | Create | Theme utilities (CSS var helpers) |
| src/index.css | Modify | Add @property for reactivity |
| src/features/accounting/routes/AccountingPage.tsx | Modify | Replace hardcoded button colors with theme colors |
| src/app/layouts/MainLayout.tsx | Modify | Add Settings nav link + icon |
| src/app/layouts/MainLayout.tsx | Modify | Use theme colors in sidebar logo highlight |

### New Files

#### src/theme/theme.ts

```typescript
// Constants for theme keys
export const THEME_KEYS = {
  PRIMARY: "primary",
  SECONDARY: "secondary",
  TERTIARY: "tertiary",
} as const;

// Theme colors (Tailwind palette)
export const THEME_COLORS = [
  "slate", "gray", "zinc", "neutral", "stone",
  "red", "orange", "amber", "yellow", "lime", "green",
  "emerald", "teal", "cyan", "sky", "blue",
  "indigo", "violet", "purple", "fuchsia", "pink", "rose"
] as const;

export type ThemeColors = typeof THEME_COLORS[number];

// Default theme
export const DEFAULT_THEME = {
  primary: "blue",
  secondary: "slate",
  tertiary: "emerald",
} as const;

// Load theme from localStorage
export function loadTheme(): Record<string, string> {
  if (typeof window === "undefined") return DEFAULT_THEME;
  const stored = localStorage.getItem("academix-theme");
  return stored ? JSON.parse(stored) : DEFAULT_THEME;
}

// Save theme to localStorage
export function saveTheme(theme: Record<string, string>): void {
  localStorage.setItem("academix-theme", JSON.stringify(theme));
}

// Apply theme to CSS
export function applyTheme(theme: Record<string, string>): void {
  const root = document.documentElement;
  Object.entries(theme).forEach(([key, value]) => {
    root.style.setProperty(`--color-${key}`, `var(--color-${value}-500)`);
  });
}
```

#### src/app/pages/settings/SettingsPage.tsx

- Page with 3 sections: Primary, Secondary, Tertiary
- Each uses grid of 5 shades (400-900) per color family
- Selected color shows checkmark
- Changes apply immediately

### Modified Files

#### src/index.css

```css
@import "tailwindcss";

@property --color-primary {
  syntax: "<color>";
  inherits: true;
  initial-value: #3b82f6;
}

@property --color-secondary {
  syntax: "<color>";
  inherits: true;
  initial-value: #64748b;
}

@property --color-tertiary {
  syntax: "<color>";
  inherits: true;
  initial-value: #22c55e;
}

@theme {
  --color-primary: var(--color-primary, #3b82f6);
  --color-secondary: var(--color-secondary, #64748b);
  --color-tertiary: var(--color-tertiary, #22c55e);
}
```

#### src/app/layouts/MainLayout.tsx

- Add SettingsIcon and navigation item
- Add to allNavigation array
- Add case "settings" to renderPage()

#### src/features/accounting/routes/AccountingPage.tsx

- Replace bg-green-600, bg-red-600, bg-blue-600 with theme CSS variables
- Example: use bg-[var(--color-primary)] instead of hardcoded colors

---

## Implementation Notes

1. Theme applies on app mount via useEffect in App.tsx
2. Color picker shows all THEME_COLORS with 5 shades each (400,500,600,700,800)
3. MainLayout needs SettingsPage import when created
4. AccountingPage buttons use theme CSS vars for bg color

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| CSS var not reactive | Use @property for explicit type |
| localStorage unavailable | Catch error, use defaults |
| Color not in palette | Use nearest Tailwind color |