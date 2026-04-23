// Compact color picker with palette and hex input

import { useState, useEffect } from "react";
import { THEME_COLORS, COLOR_SHADES, loadTheme, saveTheme, applyTheme } from "../../theme/theme";

const SHADES = [400, 500, 600, 700, 800] as const;

interface ColorPickerProps {
  label: string;
  colorKey: "primary" | "secondary" | "tertiary" | "background" | "foreground";
  description: string;
}

export function ColorPicker({ label, colorKey, description }: ColorPickerProps) {
  const [theme, setTheme] = useState<Record<string, string>>({});
  const [hexInput, setHexInput] = useState("");
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    const loaded = loadTheme();
    setTheme(loaded);
    setHexInput(loaded[colorKey] || "#3b82f6");
  }, [colorKey]);

  const currentColor = theme[colorKey] || "#3b82f6";

  const handlePaletteSelect = (hex: string) => {
    const updated = { ...theme, [colorKey]: hex };
    setTheme(updated);
    setHexInput(hex);
    saveTheme(updated);
    applyTheme(updated);
  };

  const handleHexChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setHexInput(value);
    
    // Validate hex
    if (/^#[0-9A-Fa-f]{6}$/.test(value)) {
      const updated = { ...theme, [colorKey]: value };
      setTheme(updated);
      saveTheme(updated);
      applyTheme(updated);
    }
  };

  return (
    <div className="rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="font-semibold text-[var(--color-foreground)]">{label}</h3>
          <p className="text-xs text-[var(--color-foreground)]/60">{description}</p>
        </div>
        <div
          className="h-10 w-10 rounded-lg border-2 border-[var(--color-foreground)]/20"
          style={{ backgroundColor: currentColor }}
        />
      </div>

      {/* Compact grid - single row of colors */}
      <div className="mt-3">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex w-full items-center justify-between rounded-md border border-[var(--color-foreground)]/30 px-3 py-2 text-sm"
        >
          <span className="font-mono">{currentColor}</span>
          <span className="text-[var(--color-foreground)]/60">{isOpen ? "▲" : "▼"}</span>
        </button>
        
        {isOpen && (
          <div className="mt-2 max-h-48 overflow-y-auto rounded-md border border-[var(--color-foreground)]/20 p-2">
            <div className="grid grid-cols-11 gap-1">
              {THEME_COLORS.map((colorName) => {
                const shades = COLOR_SHADES[colorName];
                if (!shades) return null;
                return SHADES.map((shade) => {
                  const hex = shades[String(shade)];
                  if (!hex) return null;
                  return (
                    <button
                      key={`${colorName}-${shade}`}
                      onClick={() => handlePaletteSelect(hex)}
                      title={`${colorName}-${shade}`}
                      className={`h-6 w-6 rounded-md transition-all ${
                        currentColor.toLowerCase() === hex.toLowerCase()
                          ? "ring-2 ring-slate-900 ring-offset-1"
                          : "hover:scale-110"
                      }`}
                      style={{ backgroundColor: hex }}
                    />
                  );
                });
              })}
            </div>
          </div>
        )}
      </div>

      {/* Hex input */}
      <div className="mt-3 flex items-center gap-2">
        <label className="text-xs font-medium text-[var(--color-foreground)]/80">Hex:</label>
        <input
          type="text"
          value={hexInput}
          onChange={handleHexChange}
          placeholder="#000000"
          maxLength={7}
          className="flex-1 rounded-md border border-[var(--color-foreground)]/30 px-2 py-1 text-sm font-mono uppercase"
        />
      </div>
    </div>
  );
}