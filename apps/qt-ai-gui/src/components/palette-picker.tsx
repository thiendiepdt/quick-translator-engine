import { Check } from "lucide-react";

import { PALETTES, type Palette } from "@/lib/theme";
import { cn } from "@/lib/utils";

interface Props {
  value: Palette;
  onChange: (palette: Palette) => void;
}

export function PalettePicker({ value, onChange }: Props) {
  return (
    <div role="radiogroup" aria-label="Bộ màu" className="grid grid-cols-3 gap-3">
      {PALETTES.map((palette) => {
        const selected = palette.id === value;
        return (
          <button
            key={palette.id}
            type="button"
            role="radio"
            aria-checked={selected}
            onClick={() => onChange(palette.id)}
            className={cn(
              "rounded-lg border p-3 text-left transition-colors hover:bg-accent/50",
              selected && "border-primary ring-2 ring-primary/30",
            )}
          >
            <div className="mb-2 grid grid-cols-2 gap-1">
              {(["light", "dark"] as const).map((mode) => {
                const [bg, fg, accent] = palette.preview[mode];
                return (
                  <div
                    key={mode}
                    className="flex h-10 items-center gap-1 rounded-md border px-2"
                    style={{ background: bg, color: fg }}
                    aria-hidden
                  >
                    <span className="size-3 rounded-full" style={{ background: accent }} />
                    <span className="h-1.5 flex-1 rounded-full" style={{ background: fg, opacity: 0.7 }} />
                  </div>
                );
              })}
            </div>
            <div className="flex items-center gap-1.5">
              <span className="text-sm font-medium">{palette.name}</span>
              {selected && <Check className="size-4 text-primary" />}
            </div>
            <p className="mt-0.5 text-xs text-muted-foreground">{palette.description}</p>
          </button>
        );
      })}
    </div>
  );
}
