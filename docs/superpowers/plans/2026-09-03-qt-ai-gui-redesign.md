# QT AI Translator — Redesign UI + Theme Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Thiết kế lại toàn bộ lớp giao diện `apps/qt-ai-gui` theo spec `docs/superpowers/specs/2026-09-03-qt-ai-gui-redesign-design.md`: 3 bộ màu × sáng/tối/hệ thống, rail trái + 4 trang, bỏ dialog lớn; giữ nguyên logic store/api/core.

**Architecture:** Token CSS viết lại theo 6 tổ hợp (palette × mode) đánh dấu bằng comment `/* palette: <id> <mode> */` để test parse được; `data-palette` trên `<html>` do `lib/theme.ts` gắn, class `dark` do next-themes. Store thêm `page`; `App` render `AppRail` + trang theo `page`. Mỗi trang là một component trong `components/pages/`, dùng lại api/store/story-form hiện có. Rust chỉ thêm 2 trường config và command `recent_summaries`.

**Tech Stack:** Tauri 2, React 19, Tailwind v4 (`@theme inline`), shadcn (radix-ui), zustand, react-hook-form + zod, next-themes, `@fontsource-variable/{inter,noto-serif,jetbrains-mono}` 5.3.0, vitest + testing-library.

## Global Constraints

- Component **chỉ dùng token** (`bg-background`, `text-muted-foreground`, `bg-status-done/10`…); cấm class màu cứng (`bg-amber-50`, `text-zinc-100`, `bg-zinc-950`, `text-amber-700`).
- Token bắt buộc cho mỗi tổ hợp palette × mode: `--background --foreground --card --card-foreground --popover --popover-foreground --primary --primary-foreground --secondary --secondary-foreground --muted --muted-foreground --accent --accent-foreground --destructive --border --input --ring --status-done --status-error --status-warning --status-queued --status-translating --log-bg --log-fg --font-reading --radius`.
- Tương phản ≥ 4.5:1 cho `foreground/background`, `muted-foreground/background`, `primary-foreground/primary`, `card-foreground/card`, `log-fg/log-bg` ở mọi tổ hợp (kiểm bằng test).
- Font offline qua `@fontsource-variable`, không tải mạng (CSP `font-src 'self' data:`).
- Không đổi `api.ts` contract cũ (chỉ thêm), không đổi `schema.ts` cũ (chỉ thêm), không đổi `story-form.ts`, không đụng `crates/qt-ai-core`.
- Bán kính: Editorial `0.375rem`, Studio `0.25rem`, Soft `0.75rem`.
- Chữ tiếng Việt cho mọi label; không dùng "mày/tao" trong UI.
- Mỗi task kết thúc bằng `npm run -s typecheck && npm run -s lint && npx vitest run` xanh (0 error lint) rồi commit.

## File Structure

```
apps/qt-ai-gui/
  package.json                      (+ 3 gói fontsource)
  src/index.css                     VIẾT LẠI: @theme inline + 6 block token + base
  src/main.tsx                      import font, boot palette, ThemeProvider system
  src/app.tsx                       shell: agy → picker → rail + page
  src/lib/theme.ts                  Palette/ThemeMode, PALETTES, applyPalette, bootPalette, rememberPalette
  src/lib/theme.test.ts
  src/lib/theme-tokens.ts           parsePaletteBlocks, parseOklch, luminance, contrastRatio, REQUIRED_TOKENS
  src/lib/theme-tokens.test.ts      coverage + contrast trên src/index.css thật
  src/lib/chapters.ts               ChapterFilter, filterChapters(rows, filter, query), FILTER_LABELS
  src/lib/chapters.test.ts
  src/lib/schema.ts                 (+ palette/themeMode trong appConfigSchema, recentSummarySchema)
  src/lib/types.ts                  (+ RecentSummary)
  src/lib/api.ts                    (+ recentSummaries)
  src/store/story.ts                (+ page/setPage, statusFilter: ChapterFilter, searchQuery)
  src/store/story.test.ts           (+ test page)
  src/hooks/use-theme.ts            useThemeSync (config → DOM), useThemeActions (setPalette/setMode → config)
  src/components/app-rail.tsx
  src/components/agy-missing.tsx    restyle
  src/components/story-picker.tsx   restyle + recent summaries
  src/components/pages/translate-page.tsx
  src/components/translate-toolbar.tsx
  src/components/chapter-list.tsx  (+ chapter-list.test.tsx)
  src/components/chapter-reader.tsx
  src/components/log-panel.tsx      restyle (token)
  src/components/pages/story-page.tsx
  src/components/glossary-editor.tsx (+ tìm nhanh)
  src/components/check-rules-editor.tsx (giữ)
  src/components/ai-fill-dialog.tsx (token hoá)
  src/components/pages/export-page.tsx
  src/components/pages/settings-page.tsx
  src/components/palette-picker.tsx
  XOÁ: workbench.tsx, progress-header.tsx, chapter-table.tsx, chapter-table.test.tsx, chapter-panel.tsx,
       story-form.tsx, settings-dialog.tsx, export-dialog.tsx
  src-tauri/src/app_config.rs       (+ palette, theme_mode)
  src-tauri/src/story_cmds.rs       (+ RecentSummary, summarize_recent, recent_summaries)
  src-tauri/src/lib.rs              (+ handler)
```

---

### Task 1: Token theme + font + test coverage/tương phản

**Files:**
- Modify: `apps/qt-ai-gui/package.json`, `apps/qt-ai-gui/src/index.css`, `apps/qt-ai-gui/src/main.tsx`
- Create: `src/lib/theme.ts`, `src/lib/theme.test.ts`, `src/lib/theme-tokens.ts`, `src/lib/theme-tokens.test.ts`

**Interfaces:**
- Produces: `theme.ts`: `PALETTE_IDS`, `type Palette`, `THEME_MODES`, `type ThemeMode`, `PALETTES: PaletteInfo[]`, `PALETTE_STORAGE_KEY = "qt-ai-palette"`, `THEME_STORAGE_KEY = "qt-ai-theme"`, `isPalette(v): v is Palette`, `isThemeMode(v): v is ThemeMode`, `applyPalette(root: HTMLElement, p: Palette)`, `bootPalette(root, storage): Palette`, `rememberPalette(storage, p)`.
- `theme-tokens.ts`: `REQUIRED_TOKENS: string[]`, `parsePaletteBlocks(css): Map<string, Record<string,string>>` (key `"<palette>/<mode>"`), `parseOklch(value): {l,c,h} | null`, `relativeLuminance(oklch)`, `contrastRatio(a, b)`.
- CSS: class tailwind mới `bg-status-done`, `text-status-error`, `bg-log`, `text-log-foreground`, `font-reading` (qua `@theme inline`).

- [ ] **Step 1: Cài font**

```bash
cd apps/qt-ai-gui && npm install @fontsource-variable/inter@5.3.0 @fontsource-variable/noto-serif@5.3.0 @fontsource-variable/jetbrains-mono@5.3.0
```

- [ ] **Step 2: Test `src/lib/theme.test.ts`**

```ts
import { describe, expect, it } from "vitest";

import { applyPalette, bootPalette, isPalette, isThemeMode, PALETTES, rememberPalette } from "@/lib/theme";

function storage(initial: Record<string, string> = {}) {
  const map = new Map(Object.entries(initial));
  return {
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => void map.set(k, v),
    map,
  };
}

describe("theme", () => {
  it("guard kiểu", () => {
    expect(isPalette("studio")).toBe(true);
    expect(isPalette("neon")).toBe(false);
    expect(isThemeMode("system")).toBe(true);
    expect(isThemeMode("auto")).toBe(false);
    expect(PALETTES.map((p) => p.id)).toEqual(["editorial", "studio", "soft"]);
  });

  it("applyPalette gắn data-palette", () => {
    const root = document.createElement("html");
    applyPalette(root, "soft");
    expect(root.dataset.palette).toBe("soft");
  });

  it("bootPalette đọc cache, sai thì về editorial; rememberPalette ghi cache", () => {
    const root = document.createElement("html");
    expect(bootPalette(root, storage({ "qt-ai-palette": "studio" }))).toBe("studio");
    expect(root.dataset.palette).toBe("studio");
    expect(bootPalette(root, storage({ "qt-ai-palette": "rác" }))).toBe("editorial");
    const s = storage();
    rememberPalette(s, "soft");
    expect(s.map.get("qt-ai-palette")).toBe("soft");
  });
});
```

- [ ] **Step 3: Test `src/lib/theme-tokens.test.ts`**

```ts
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { PALETTE_IDS } from "@/lib/theme";
import { contrastRatio, parseOklch, parsePaletteBlocks, relativeLuminance, REQUIRED_TOKENS } from "@/lib/theme-tokens";

const css = readFileSync(resolve(import.meta.dirname, "../index.css"), "utf8");
const blocks = parsePaletteBlocks(css);

describe("theme tokens", () => {
  it("parseOklch + luminance: trắng ≈ 1, đen ≈ 0", () => {
    expect(parseOklch("oklch(1 0 0)")).toEqual({ l: 1, c: 0, h: 0 });
    expect(relativeLuminance({ l: 1, c: 0, h: 0 })).toBeCloseTo(1, 2);
    expect(relativeLuminance({ l: 0, c: 0, h: 0 })).toBeCloseTo(0, 3);
    expect(contrastRatio({ l: 1, c: 0, h: 0 }, { l: 0, c: 0, h: 0 })).toBeCloseTo(21, 0);
    expect(parseOklch("red")).toBeNull();
  });

  it.each(PALETTE_IDS.flatMap((p) => [`${p}/light`, `${p}/dark`]))("%s có đủ token bắt buộc", (key) => {
    const tokens = blocks.get(key);
    expect(tokens, `thiếu block ${key}`).toBeDefined();
    const missing = REQUIRED_TOKENS.filter((t) => !(t in tokens!));
    expect(missing).toEqual([]);
  });

  const PAIRS: Array<[string, string]> = [
    ["--foreground", "--background"],
    ["--muted-foreground", "--background"],
    ["--card-foreground", "--card"],
    ["--primary-foreground", "--primary"],
    ["--log-fg", "--log-bg"],
  ];

  it.each([...blocks.keys()])("%s tương phản ≥ 4.5", (key) => {
    const tokens = blocks.get(key)!;
    for (const [fg, bg] of PAIRS) {
      const a = parseOklch(tokens[fg] ?? "");
      const b = parseOklch(tokens[bg] ?? "");
      expect(a, `${key} ${fg} không phải oklch`).not.toBeNull();
      expect(b, `${key} ${bg} không phải oklch`).not.toBeNull();
      const ratio = contrastRatio(a!, b!);
      expect(ratio, `${key}: ${fg} trên ${bg} = ${ratio.toFixed(2)}`).toBeGreaterThanOrEqual(4.5);
    }
  });
});
```

- [ ] **Step 4: Chạy để thấy fail**

Run: `cd apps/qt-ai-gui && npx vitest run src/lib/theme.test.ts src/lib/theme-tokens.test.ts`
Expected: FAIL — module chưa có.

- [ ] **Step 5: `src/lib/theme.ts`**

```ts
export const PALETTE_IDS = ["editorial", "studio", "soft"] as const;
export type Palette = (typeof PALETTE_IDS)[number];
export const THEME_MODES = ["light", "dark", "system"] as const;
export type ThemeMode = (typeof THEME_MODES)[number];

export const PALETTE_STORAGE_KEY = "qt-ai-palette";
export const THEME_STORAGE_KEY = "qt-ai-theme";
export const DEFAULT_PALETTE: Palette = "editorial";

export interface PaletteInfo {
  id: Palette;
  name: string;
  description: string;
  /** Màu xem trước cho thẻ chọn (không cần khớp token tuyệt đối). */
  preview: { light: [string, string, string]; dark: [string, string, string] };
}

export const PALETTES: PaletteInfo[] = [
  {
    id: "editorial",
    name: "Editorial",
    description: "Giấy ấm, bản dịch chữ serif, nhấn đất nung. Dành cho việc đọc.",
    preview: { light: ["#fbf7ef", "#2b251e", "#9a4b2a"], dark: ["#201b17", "#ece3d4", "#d9865c"] },
  },
  {
    id: "studio",
    name: "Studio",
    description: "Graphite, nhấn teal, số liệu monospace, mật độ cao.",
    preview: { light: ["#ffffff", "#0f172a", "#0f8f84"], dark: ["#0e131a", "#e6edf3", "#2dd4bf"] },
  },
  {
    id: "soft",
    name: "Soft",
    description: "Bo tròn, tím dịu, nhiều khoảng trắng, thân thiện.",
    preview: { light: ["#fdfcff", "#1f1b2e", "#6d3fd6"], dark: ["#18151f", "#ece9f7", "#a78bfa"] },
  },
];

export const THEME_MODE_LABELS: Record<ThemeMode, string> = { light: "Sáng", dark: "Tối", system: "Theo hệ thống" };

export function isPalette(value: unknown): value is Palette {
  return typeof value === "string" && (PALETTE_IDS as readonly string[]).includes(value);
}

export function isThemeMode(value: unknown): value is ThemeMode {
  return typeof value === "string" && (THEME_MODES as readonly string[]).includes(value);
}

export function applyPalette(root: HTMLElement, palette: Palette) {
  root.dataset.palette = palette;
}

type StorageLike = Pick<Storage, "getItem" | "setItem">;

/** Gọi trước khi render: áp bộ màu đã cache để không nháy; AppConfig sẽ ghi đè khi nạp xong. */
export function bootPalette(root: HTMLElement, storage: Pick<Storage, "getItem">): Palette {
  const cached = storage.getItem(PALETTE_STORAGE_KEY);
  const palette = isPalette(cached) ? cached : DEFAULT_PALETTE;
  applyPalette(root, palette);
  return palette;
}

export function rememberPalette(storage: StorageLike, palette: Palette) {
  storage.setItem(PALETTE_STORAGE_KEY, palette);
}
```

- [ ] **Step 6: `src/lib/theme-tokens.ts`**

```ts
/** Đọc token từ index.css để test coverage + tương phản. Block bắt đầu bằng comment `/* palette: <id> <mode> *\/`. */

export const REQUIRED_TOKENS = [
  "--background", "--foreground", "--card", "--card-foreground", "--popover", "--popover-foreground",
  "--primary", "--primary-foreground", "--secondary", "--secondary-foreground", "--muted", "--muted-foreground",
  "--accent", "--accent-foreground", "--destructive", "--border", "--input", "--ring",
  "--status-done", "--status-error", "--status-warning", "--status-queued", "--status-translating",
  "--log-bg", "--log-fg", "--font-reading", "--radius",
];

const MARKER = /\/\*\s*palette:\s*([a-z]+)\s+(light|dark)\s*\*\//g;

export function parsePaletteBlocks(css: string): Map<string, Record<string, string>> {
  const result = new Map<string, Record<string, string>>();
  const markers = [...css.matchAll(MARKER)];
  markers.forEach((match, index) => {
    const start = match.index + match[0].length;
    const end = index + 1 < markers.length ? markers[index + 1]!.index : css.length;
    const body = css.slice(start, end);
    const tokens: Record<string, string> = {};
    for (const line of body.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)) {
      tokens[line[1]!] = line[2]!.trim();
    }
    result.set(`${match[1]}/${match[2]}`, tokens);
  });
  return result;
}

export interface Oklch {
  l: number;
  c: number;
  h: number;
}

export function parseOklch(value: string): Oklch | null {
  const match = /oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)/.exec(value);
  if (!match) return null;
  return { l: Number(match[1]), c: Number(match[2]), h: Number(match[3]) };
}

function clamp01(x: number) {
  return Math.min(1, Math.max(0, x));
}

/** OKLCH → linear sRGB (Björn Ottosson) → luminance tương đối WCAG. */
export function relativeLuminance({ l, c, h }: Oklch): number {
  const rad = (h * Math.PI) / 180;
  const a = c * Math.cos(rad);
  const b = c * Math.sin(rad);
  const l_ = l + 0.3963377774 * a + 0.2158037573 * b;
  const m_ = l - 0.1055613458 * a - 0.0638541728 * b;
  const s_ = l - 0.0894841775 * a - 1.291485548 * b;
  const L = l_ ** 3;
  const M = m_ ** 3;
  const S = s_ ** 3;
  const r = clamp01(4.0767416621 * L - 3.3077115913 * M + 0.2309699292 * S);
  const g = clamp01(-1.2684380046 * L + 2.6097574011 * M - 0.3413193965 * S);
  const bl = clamp01(-0.0041960863 * L - 0.7034186147 * M + 1.707614701 * S);
  return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
}

export function contrastRatio(a: Oklch, b: Oklch): number {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  const [hi, lo] = la > lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}
```

- [ ] **Step 7: Viết lại `src/index.css`**

```css
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:is(.dark *));

@theme inline {
  --font-sans: var(--font-ui);
  --font-serif: var(--font-reading);
  --font-mono: var(--font-code);
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-card-foreground: var(--card-foreground);
  --color-popover: var(--popover);
  --color-popover-foreground: var(--popover-foreground);
  --color-primary: var(--primary);
  --color-primary-foreground: var(--primary-foreground);
  --color-secondary: var(--secondary);
  --color-secondary-foreground: var(--secondary-foreground);
  --color-muted: var(--muted);
  --color-muted-foreground: var(--muted-foreground);
  --color-accent: var(--accent);
  --color-accent-foreground: var(--accent-foreground);
  --color-destructive: var(--destructive);
  --color-border: var(--border);
  --color-input: var(--input);
  --color-ring: var(--ring);
  --color-status-done: var(--status-done);
  --color-status-error: var(--status-error);
  --color-status-warning: var(--status-warning);
  --color-status-queued: var(--status-queued);
  --color-status-translating: var(--status-translating);
  --color-log: var(--log-bg);
  --color-log-foreground: var(--log-fg);
  --radius-sm: calc(var(--radius) - 4px);
  --radius-md: calc(var(--radius) - 2px);
  --radius-lg: var(--radius);
  --radius-xl: calc(var(--radius) + 4px);
}

/* Font chung cho mọi bộ; --font-reading đổi theo bộ màu. */
:root {
  --font-ui: "Inter Variable", "Segoe UI", system-ui, sans-serif;
  --font-code: "JetBrains Mono Variable", Consolas, ui-monospace, monospace;
  --reading-size: 1rem;
}

/* ============ EDITORIAL ============ */
/* palette: editorial light */
:root,
:root[data-palette="editorial"] {
  --radius: 0.375rem;
  --font-reading: "Noto Serif Variable", Georgia, serif;
  --background: oklch(0.975 0.012 85);
  --foreground: oklch(0.27 0.02 60);
  --card: oklch(0.99 0.008 85);
  --card-foreground: oklch(0.27 0.02 60);
  --popover: oklch(0.99 0.008 85);
  --popover-foreground: oklch(0.27 0.02 60);
  --primary: oklch(0.47 0.13 40);
  --primary-foreground: oklch(0.98 0.01 85);
  --secondary: oklch(0.93 0.02 85);
  --secondary-foreground: oklch(0.32 0.02 60);
  --muted: oklch(0.94 0.015 85);
  --muted-foreground: oklch(0.46 0.02 65);
  --accent: oklch(0.91 0.03 80);
  --accent-foreground: oklch(0.30 0.03 50);
  --destructive: oklch(0.52 0.19 28);
  --border: oklch(0.88 0.02 85);
  --input: oklch(0.88 0.02 85);
  --ring: oklch(0.60 0.12 40);
  --status-done: oklch(0.55 0.12 145);
  --status-error: oklch(0.55 0.19 28);
  --status-warning: oklch(0.68 0.15 75);
  --status-queued: oklch(0.72 0.02 85);
  --status-translating: oklch(0.58 0.12 250);
  --log-bg: oklch(0.22 0.012 60);
  --log-fg: oklch(0.90 0.02 85);
}

/* palette: editorial dark */
.dark,
.dark[data-palette="editorial"] {
  --background: oklch(0.20 0.012 60);
  --foreground: oklch(0.92 0.02 85);
  --card: oklch(0.24 0.014 60);
  --card-foreground: oklch(0.92 0.02 85);
  --popover: oklch(0.24 0.014 60);
  --popover-foreground: oklch(0.92 0.02 85);
  --primary: oklch(0.74 0.13 45);
  --primary-foreground: oklch(0.20 0.02 50);
  --secondary: oklch(0.29 0.015 60);
  --secondary-foreground: oklch(0.90 0.02 85);
  --muted: oklch(0.27 0.012 60);
  --muted-foreground: oklch(0.72 0.02 75);
  --accent: oklch(0.32 0.03 55);
  --accent-foreground: oklch(0.94 0.02 85);
  --destructive: oklch(0.66 0.17 28);
  --border: oklch(0.32 0.015 60);
  --input: oklch(0.32 0.015 60);
  --ring: oklch(0.74 0.12 45);
  --status-done: oklch(0.72 0.13 145);
  --status-error: oklch(0.68 0.18 28);
  --status-warning: oklch(0.78 0.14 80);
  --status-queued: oklch(0.45 0.015 60);
  --status-translating: oklch(0.72 0.12 250);
  --log-bg: oklch(0.15 0.01 60);
  --log-fg: oklch(0.88 0.02 85);
}

/* ============ STUDIO ============ */
/* palette: studio light */
:root[data-palette="studio"] {
  --radius: 0.25rem;
  --font-reading: var(--font-ui);
  --background: oklch(1 0 0);
  --foreground: oklch(0.20 0.02 255);
  --card: oklch(0.985 0.003 250);
  --card-foreground: oklch(0.20 0.02 255);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.20 0.02 255);
  --primary: oklch(0.46 0.09 185);
  --primary-foreground: oklch(0.99 0 0);
  --secondary: oklch(0.95 0.006 250);
  --secondary-foreground: oklch(0.30 0.02 255);
  --muted: oklch(0.96 0.005 250);
  --muted-foreground: oklch(0.46 0.02 255);
  --accent: oklch(0.94 0.02 185);
  --accent-foreground: oklch(0.28 0.05 190);
  --destructive: oklch(0.55 0.20 25);
  --border: oklch(0.90 0.008 250);
  --input: oklch(0.90 0.008 250);
  --ring: oklch(0.60 0.10 185);
  --status-done: oklch(0.55 0.14 150);
  --status-error: oklch(0.55 0.20 25);
  --status-warning: oklch(0.70 0.15 70);
  --status-queued: oklch(0.75 0.01 250);
  --status-translating: oklch(0.55 0.15 250);
  --log-bg: oklch(0.17 0.012 255);
  --log-fg: oklch(0.90 0.01 250);
}

/* palette: studio dark */
.dark[data-palette="studio"] {
  --background: oklch(0.15 0.012 255);
  --foreground: oklch(0.93 0.01 250);
  --card: oklch(0.19 0.014 255);
  --card-foreground: oklch(0.93 0.01 250);
  --popover: oklch(0.19 0.014 255);
  --popover-foreground: oklch(0.93 0.01 250);
  --primary: oklch(0.80 0.12 185);
  --primary-foreground: oklch(0.17 0.03 190);
  --secondary: oklch(0.25 0.014 255);
  --secondary-foreground: oklch(0.90 0.01 250);
  --muted: oklch(0.23 0.012 255);
  --muted-foreground: oklch(0.70 0.015 250);
  --accent: oklch(0.28 0.03 190);
  --accent-foreground: oklch(0.93 0.01 250);
  --destructive: oklch(0.68 0.18 25);
  --border: oklch(0.28 0.014 255);
  --input: oklch(0.28 0.014 255);
  --ring: oklch(0.80 0.12 185);
  --status-done: oklch(0.75 0.15 150);
  --status-error: oklch(0.70 0.18 25);
  --status-warning: oklch(0.80 0.14 80);
  --status-queued: oklch(0.45 0.012 255);
  --status-translating: oklch(0.72 0.14 250);
  --log-bg: oklch(0.11 0.01 255);
  --log-fg: oklch(0.90 0.01 250);
}

/* ============ SOFT ============ */
/* palette: soft light */
:root[data-palette="soft"] {
  --radius: 0.75rem;
  --font-reading: var(--font-ui);
  --background: oklch(0.99 0.004 300);
  --foreground: oklch(0.22 0.03 290);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.22 0.03 290);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.22 0.03 290);
  --primary: oklch(0.48 0.20 295);
  --primary-foreground: oklch(0.99 0 0);
  --secondary: oklch(0.95 0.02 295);
  --secondary-foreground: oklch(0.35 0.06 295);
  --muted: oklch(0.96 0.012 295);
  --muted-foreground: oklch(0.47 0.03 290);
  --accent: oklch(0.93 0.04 300);
  --accent-foreground: oklch(0.35 0.10 295);
  --destructive: oklch(0.58 0.20 15);
  --border: oklch(0.92 0.015 295);
  --input: oklch(0.92 0.015 295);
  --ring: oklch(0.65 0.18 295);
  --status-done: oklch(0.58 0.13 150);
  --status-error: oklch(0.58 0.20 15);
  --status-warning: oklch(0.72 0.14 70);
  --status-queued: oklch(0.78 0.01 295);
  --status-translating: oklch(0.60 0.15 260);
  --log-bg: oklch(0.20 0.02 290);
  --log-fg: oklch(0.92 0.01 300);
}

/* palette: soft dark */
.dark[data-palette="soft"] {
  --background: oklch(0.17 0.02 290);
  --foreground: oklch(0.94 0.01 300);
  --card: oklch(0.21 0.025 290);
  --card-foreground: oklch(0.94 0.01 300);
  --popover: oklch(0.21 0.025 290);
  --popover-foreground: oklch(0.94 0.01 300);
  --primary: oklch(0.78 0.13 300);
  --primary-foreground: oklch(0.18 0.04 295);
  --secondary: oklch(0.27 0.03 290);
  --secondary-foreground: oklch(0.92 0.01 300);
  --muted: oklch(0.25 0.025 290);
  --muted-foreground: oklch(0.72 0.03 295);
  --accent: oklch(0.30 0.05 295);
  --accent-foreground: oklch(0.95 0.01 300);
  --destructive: oklch(0.70 0.17 15);
  --border: oklch(0.30 0.03 290);
  --input: oklch(0.30 0.03 290);
  --ring: oklch(0.78 0.13 300);
  --status-done: oklch(0.75 0.13 150);
  --status-error: oklch(0.72 0.17 15);
  --status-warning: oklch(0.80 0.13 80);
  --status-queued: oklch(0.45 0.02 290);
  --status-translating: oklch(0.74 0.13 260);
  --log-bg: oklch(0.13 0.015 290);
  --log-fg: oklch(0.92 0.01 300);
}

@layer base {
  * {
    @apply border-border outline-ring/50;
  }

  html,
  body,
  #root {
    height: 100%;
  }

  body {
    @apply bg-background text-foreground font-sans antialiased;
    margin: 0;
    overflow: hidden;
  }

  button:not(:disabled),
  [role="button"]:not([aria-disabled="true"]) {
    cursor: pointer;
  }

  ::selection {
    background: color-mix(in oklch, var(--primary) 25%, transparent);
  }
}

@layer components {
  /* Vùng đọc bản dịch: cột hẹp, chữ to, giãn dòng. */
  .reading {
    font-family: var(--font-reading);
    font-size: var(--reading-size);
    line-height: 1.75;
    max-width: 70ch;
  }

  .fine-scrollbar {
    scrollbar-color: color-mix(in oklch, var(--foreground) 25%, transparent) transparent;
    scrollbar-width: thin;
  }
}
```

- [ ] **Step 8: `src/main.tsx`**

```tsx
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "@fontsource-variable/noto-serif";

import { ThemeProvider } from "next-themes";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import App from "@/app";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { bootPalette, THEME_STORAGE_KEY } from "@/lib/theme";

import "@/index.css";

// Áp bộ màu cache trước khi render để không nháy; AppConfig ghi đè khi nạp (hooks/use-theme).
bootPalette(document.documentElement, localStorage);

const root = document.getElementById("root");
if (!root) throw new Error("Thiếu #root");

createRoot(root).render(
  <StrictMode>
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem storageKey={THEME_STORAGE_KEY} disableTransitionOnChange>
      <TooltipProvider>
        <App />
        <Toaster position="top-right" richColors />
      </TooltipProvider>
    </ThemeProvider>
  </StrictMode>,
);
```

Kiểm `src/components/ui/sonner.tsx` dùng `useTheme` từ next-themes (bản shadcn mặc định có) — nếu có, toast đổi theo mode tự động; không cần sửa.

- [ ] **Step 9: Chạy test + check**

Run: `npx vitest run && npm run -s typecheck && npm run -s lint && npm run -s build`
Expected: theme.test 3 pass, theme-tokens: 1 + 6 coverage + 6 contrast pass; nếu một cặp tương phản < 4.5, **chỉnh L của token đó** (tăng L của *-foreground sáng hoặc giảm L của nền) rồi chạy lại; build xanh (font được bundle vào `dist/assets/*.woff2`).

- [ ] **Step 10: Commit**

```bash
git add apps/qt-ai-gui/package.json apps/qt-ai-gui/package-lock.json apps/qt-ai-gui/src/index.css apps/qt-ai-gui/src/main.tsx apps/qt-ai-gui/src/lib/theme.ts apps/qt-ai-gui/src/lib/theme.test.ts apps/qt-ai-gui/src/lib/theme-tokens.ts apps/qt-ai-gui/src/lib/theme-tokens.test.ts
git commit -m "feat(qt-ai-gui): hệ token 3 bộ màu × sáng/tối, font offline, test coverage + tương phản token

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 2: Rust — AppConfig thêm palette/themeMode, command recent_summaries; TS schema/api

**Files:**
- Modify: `src-tauri/src/app_config.rs`, `src-tauri/src/story_cmds.rs`, `src-tauri/src/lib.rs`, `src/lib/schema.ts`, `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/schema.test.ts`

**Interfaces:**
- Rust: `AppConfig { …, palette: String /* "editorial" */, theme_mode: String /* "system" */ }` camelCase `palette`, `themeMode`. `RecentSummary { root, name: Option<String>, done: Option<usize>, total: Option<usize> }` camelCase; `summarize_recent(roots: &[String]) -> Vec<RecentSummary>`; command `recent_summaries(state) -> Vec<RecentSummary>`.
- TS: `appConfigSchema` thêm `palette: z.string().default("editorial")`, `themeMode: z.string().default("system")`; `recentSummarySchema`; `type RecentSummary`; `api.recentSummaries(): Promise<RecentSummary[]>`.

- [ ] **Step 1: Test Rust — thêm vào `app_config.rs` tests**

```rust
    #[test]
    fn default_co_palette_editorial_va_theme_system_va_doc_config_cu_thieu_truong() {
        let config = AppConfig::default();
        assert_eq!(config.palette, "editorial");
        assert_eq!(config.theme_mode, "system");
        let old: AppConfig = serde_json::from_str(r#"{"agyPath":null,"model":null,"maxSessions":7,"recent":[]}"#).unwrap();
        assert_eq!(old.max_sessions, 7);
        assert_eq!(old.palette, "editorial");
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["themeMode"], "system");
    }
```

Thêm vào `story_cmds.rs` tests:

```rust
    #[test]
    fn summarize_recent_doc_ten_va_tien_do_folder_hong_thi_none() {
        let dir = story();
        save_story_inner(dir.path(), serde_json::json!({"name": "Nam Nữ Đế"})).unwrap();
        let roots = vec![dir.path().display().to_string(), "D:\\khong\\co".to_string()];
        let list = summarize_recent(&roots);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name.as_deref(), Some("Nam Nữ Đế"));
        assert_eq!(list[0].total, Some(2));
        assert_eq!(list[0].done, Some(0));
        assert_eq!(list[1].root, "D:\\khong\\co");
        assert!(list[1].name.is_none() && list[1].total.is_none());
    }
```

- [ ] **Step 2: Chạy để thấy fail**

Run: `cargo test -p qt-ai-gui`
Expected: compile error (thiếu field/fn).

- [ ] **Step 3: Implement**

`app_config.rs` — struct và Default:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    /// Đường dẫn agy chỉnh tay; None = tự tìm trong PATH.
    pub agy_path: Option<String>,
    pub model: Option<String>,
    pub max_sessions: u32,
    /// Folder truyện mở gần đây, mới nhất đứng đầu.
    pub recent: Vec<String>,
    /// Bộ màu: editorial | studio | soft (UI kiểm tra giá trị, Rust chỉ lưu).
    pub palette: String,
    /// light | dark | system.
    pub theme_mode: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            agy_path: None,
            model: None,
            max_sessions: 50,
            recent: vec![],
            palette: "editorial".to_string(),
            theme_mode: "system".to_string(),
        }
    }
}
```

`story_cmds.rs` — thêm sau `ExportOutcome`:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentSummary {
    pub root: String,
    pub name: Option<String>,
    pub done: Option<usize>,
    pub total: Option<usize>,
}

/// Tóm tắt từng folder gần đây; folder hỏng/mất → chỉ có root (UI hiện mờ).
pub fn summarize_recent(roots: &[String]) -> Vec<RecentSummary> {
    roots
        .iter()
        .map(|root| {
            let paths = story_paths(Path::new(root));
            match (load_state(&paths), load_story_config(&paths)) {
                (Ok(state), Ok(story)) => {
                    let counts = count_chapters(&state);
                    RecentSummary {
                        root: root.clone(),
                        name: Some(story.name).filter(|n| !n.trim().is_empty()),
                        done: Some(counts.done),
                        total: Some(counts.total),
                    }
                }
                _ => RecentSummary { root: root.clone(), name: None, done: None, total: None },
            }
        })
        .collect()
}

#[tauri::command]
pub fn recent_summaries(state: State<'_, AppState>) -> CmdResult<Vec<RecentSummary>> {
    let roots = state.config.lock().unwrap().recent.clone();
    Ok(summarize_recent(&roots))
}
```

`lib.rs`: thêm `story_cmds::recent_summaries,` vào `generate_handler!` (sau `reveal_folder`).

- [ ] **Step 4: TS schema/api**

`schema.ts` — sửa `appConfigSchema` và thêm `recentSummarySchema`:

```ts
export const appConfigSchema = z.object({
  agyPath: z.string().nullable(),
  model: z.string().nullable(),
  maxSessions: z.number().int().min(1).max(1000),
  recent: z.array(z.string()),
  palette: z.string().default("editorial"),
  themeMode: z.string().default("system"),
});

export const recentSummarySchema = z.object({
  root: z.string(),
  name: z.string().nullable(),
  done: z.number().nullable(),
  total: z.number().nullable(),
});
```

`types.ts`: thêm import type `recentSummarySchema` và `export type RecentSummary = z.infer<typeof recentSummarySchema>;`.

`api.ts`: import `recentSummarySchema`, thêm:

```ts
export const recentSummaries = () =>
  call("recent_summaries", undefined, (v) => z.array(recentSummarySchema).parse(v));
```

(thêm `import { z } from "zod";` đầu file api.ts.)

`schema.test.ts` thêm case:

```ts
  it("appConfig cũ thiếu palette/themeMode vẫn parse với default", () => {
    const parsed = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });
    expect(parsed.palette).toBe("editorial");
    expect(parsed.themeMode).toBe("system");
  });
```

(import `appConfigSchema` vào test.)

- [ ] **Step 5: Kiểm**

Run: `cargo test -p qt-ai-gui && cd apps/qt-ai-gui && npx vitest run && npm run -s typecheck && npm run -s lint`
Expected: Rust 10 pass; vitest xanh.

- [ ] **Step 6: Commit**

```bash
git add apps/qt-ai-gui/src-tauri/src apps/qt-ai-gui/src/lib
git commit -m "feat(qt-ai-gui): AppConfig lưu palette/themeMode, command recent_summaries (tên + tiến độ truyện gần đây)

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 3: Store `page` + bộ lọc chương, hook theme, rail, shell App, màn agy/picker mới

**Files:**
- Create: `src/lib/chapters.ts`, `src/lib/chapters.test.ts`, `src/hooks/use-theme.ts`, `src/components/app-rail.tsx`
- Modify: `src/store/story.ts`, `src/store/story.test.ts`, `src/app.tsx`, `src/components/agy-missing.tsx`, `src/components/story-picker.tsx`

**Interfaces:**
- `chapters.ts`: `type ChapterFilter = ChapterStatus | "all" | "warning"`; `FILTER_ORDER: ChapterFilter[]`; `FILTER_LABELS: Record<ChapterFilter,string>`; `filterChapters(rows, filter, query): ChapterRow[]`; `countByFilter(rows): Record<ChapterFilter, number>`.
- Store: `page: Page`, `setPage(page)`, `statusFilter: ChapterFilter`, `searchQuery: string`, `setSearchQuery(q)`; `type Page = "translate" | "story" | "export" | "settings"`.
- `use-theme.ts`: `useThemeSync()` (gọi một lần ở App), `useThemeActions(): { palette: Palette; mode: ThemeMode; setPalette(p): Promise<void>; setMode(m): Promise<void> }`.
- `<AppRail />` đọc store; `App` render `Workbench` tạm = `<div>` cho tới Task 4 (các trang import ở Task 4–6).

- [ ] **Step 1: Test `src/lib/chapters.test.ts`**

```ts
import { describe, expect, it } from "vitest";

import { countByFilter, filterChapters } from "@/lib/chapters";
import type { ChapterRow } from "@/lib/types";

const rows: ChapterRow[] = [
  { id: "chuong-0001", status: "done", reviewRound: 0, reason: null, warnings: [] },
  { id: "chuong-0002", status: "done", reviewRound: 3, reason: null, warnings: ["[[1]] CJK"] },
  { id: "chuong-0003", status: "error", reviewRound: 3, reason: "Quá 3 vòng", warnings: [] },
  { id: "chuong-0010", status: "queued", reviewRound: 0, reason: null, warnings: [] },
];

describe("chapters", () => {
  it("lọc theo trạng thái; warning = done có cảnh báo; all giữ hết", () => {
    expect(filterChapters(rows, "all", "")).toHaveLength(4);
    expect(filterChapters(rows, "error", "").map((r) => r.id)).toEqual(["chuong-0003"]);
    expect(filterChapters(rows, "warning", "").map((r) => r.id)).toEqual(["chuong-0002"]);
    expect(filterChapters(rows, "done", "")).toHaveLength(2);
  });

  it("tìm theo mã không phân biệt hoa thường, kết hợp với lọc", () => {
    expect(filterChapters(rows, "all", "001").map((r) => r.id)).toEqual(["chuong-0001", "chuong-0010"]);
    expect(filterChapters(rows, "done", "CHUONG-0002")).toHaveLength(1);
  });

  it("countByFilter đếm từng chip", () => {
    const counts = countByFilter(rows);
    expect(counts.all).toBe(4);
    expect(counts.done).toBe(2);
    expect(counts.warning).toBe(1);
    expect(counts.skipped).toBe(0);
  });
});
```

Thêm vào `src/store/story.test.ts`:

```ts
  it("page mặc định translate, openStory reset về translate, setPage đổi", () => {
    useStoryStore.getState().setPage("settings");
    expect(useStoryStore.getState().page).toBe("settings");
    useStoryStore.getState().openStory({
      root: "D:\\t", chapters: [],
      counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
      settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
      story: { name: "", sourceUrl: "", protagonist: "", summary: "", glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} }, style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] }, customPrompt: "", checkRules: [], autoGlossaryLog: [], autoGlossary: "inherit" as const },
      sessionRunning: false,
    });
    expect(useStoryStore.getState().page).toBe("translate");
    expect(useStoryStore.getState().searchQuery).toBe("");
  });
```

- [ ] **Step 2: Chạy để thấy fail**

Run: `npx vitest run src/lib/chapters.test.ts src/store/story.test.ts` → FAIL.

- [ ] **Step 3: `src/lib/chapters.ts`**

```ts
import { STATUS_LABELS, type ChapterRow, type ChapterStatus } from "@/lib/types";

export type ChapterFilter = ChapterStatus | "all" | "warning";

export const FILTER_ORDER: ChapterFilter[] = ["all", "queued", "translating", "done", "warning", "error", "skipped"];

export const FILTER_LABELS: Record<ChapterFilter, string> = { ...STATUS_LABELS, all: "Tất cả", warning: "Cảnh báo" };

function matches(row: ChapterRow, filter: ChapterFilter): boolean {
  if (filter === "all") return true;
  if (filter === "warning") return row.status === "done" && row.warnings.length > 0;
  return row.status === filter;
}

export function filterChapters(rows: ChapterRow[], filter: ChapterFilter, query: string): ChapterRow[] {
  const needle = query.trim().toLowerCase();
  return rows.filter((row) => matches(row, filter) && (needle === "" || row.id.toLowerCase().includes(needle)));
}

export function countByFilter(rows: ChapterRow[]): Record<ChapterFilter, number> {
  const counts = Object.fromEntries(FILTER_ORDER.map((f) => [f, 0])) as Record<ChapterFilter, number>;
  for (const row of rows) {
    for (const filter of FILTER_ORDER) if (matches(row, filter)) counts[filter] += 1;
  }
  return counts;
}
```

- [ ] **Step 4: Store `src/store/story.ts` — sửa**

Thay import types và các trường liên quan:

```ts
import type { ChapterFilter } from "@/lib/chapters";
import type { AgyStatus, AppConfig, Progress, SessionEvent, StopReason, StorySnapshot } from "@/lib/types";

export type Page = "translate" | "story" | "export" | "settings";
```

Trong `interface StoryState` thay `statusFilter: ChapterStatus | "all";` bằng:

```ts
  page: Page;
  statusFilter: ChapterFilter;
  searchQuery: string;
```

và actions: `setPage: (page: Page) => void;`, `setStatusFilter: (filter: ChapterFilter) => void;`, `setSearchQuery: (query: string) => void;`.

Trong `create`: thêm `page: "translate", searchQuery: "",`; trong `openStory` set thêm `page: "translate", searchQuery: "",`; thêm `setPage: (page) => set({ page }),`, `setSearchQuery: (searchQuery) => set({ searchQuery }),`. Bỏ import `ChapterStatus` nếu không còn dùng.

- [ ] **Step 5: `src/hooks/use-theme.ts`**

```ts
import { useTheme } from "next-themes";
import { useEffect } from "react";
import { toast } from "sonner";

import { appConfigSet } from "@/lib/api";
import {
  applyPalette,
  DEFAULT_PALETTE,
  isPalette,
  isThemeMode,
  rememberPalette,
  type Palette,
  type ThemeMode,
} from "@/lib/theme";
import { useStoryStore } from "@/store/story";

/** AppConfig là nguồn sự thật: khi config nạp/đổi → áp palette lên <html> và mode vào next-themes. */
export function useThemeSync() {
  const config = useStoryStore((s) => s.config);
  const { setTheme } = useTheme();
  useEffect(() => {
    if (!config) return;
    const palette = isPalette(config.palette) ? config.palette : DEFAULT_PALETTE;
    applyPalette(document.documentElement, palette);
    rememberPalette(localStorage, palette);
    if (isThemeMode(config.themeMode)) setTheme(config.themeMode);
  }, [config, setTheme]);
}

export function useThemeActions() {
  const config = useStoryStore((s) => s.config);
  const setConfig = useStoryStore((s) => s.setConfig);
  const { theme } = useTheme();
  const palette: Palette = isPalette(config?.palette) ? config.palette : DEFAULT_PALETTE;
  const mode: ThemeMode = isThemeMode(theme) ? theme : "system";

  async function save(patch: Partial<Pick<typeof palette extends never ? never : { palette: string; themeMode: string }, "palette" | "themeMode">>) {
    if (!config) return;
    try {
      setConfig(await appConfigSet({ ...config, ...patch }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được giao diện");
    }
  }

  return {
    palette,
    mode,
    setPalette: (next: Palette) => save({ palette: next }),
    setMode: (next: ThemeMode) => save({ themeMode: next }),
    toggleMode: () => save({ themeMode: mode === "dark" ? "light" : "dark" }),
  };
}
```

Nếu kiểu của `save` gây khó đọc/lint, thay chữ ký bằng `async function save(patch: { palette?: string; themeMode?: string })`.

- [ ] **Step 6: `src/components/app-rail.tsx`**

```tsx
import { BookUser, Download, Languages, LibraryBig, Moon, Settings2, Sun } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useThemeActions } from "@/hooks/use-theme";
import { cn } from "@/lib/utils";
import { useStoryStore, type Page } from "@/store/story";

const ITEMS: Array<{ page: Page; label: string; icon: typeof Languages }> = [
  { page: "translate", label: "Dịch", icon: Languages },
  { page: "story", label: "Hồ sơ truyện", icon: BookUser },
  { page: "export", label: "Export", icon: Download },
  { page: "settings", label: "Cài đặt", icon: Settings2 },
];

function RailButton({ label, active, onClick, children }: { label: string; active?: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          aria-label={label}
          aria-current={active ? "page" : undefined}
          onClick={onClick}
          className={cn("size-10 rounded-lg text-muted-foreground hover:text-foreground", active && "bg-accent text-accent-foreground")}
        >
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent side="right">{label}</TooltipContent>
    </Tooltip>
  );
}

export function AppRail() {
  const page = useStoryStore((s) => s.page);
  const setPage = useStoryStore((s) => s.setPage);
  const running = useStoryStore((s) => s.session.status === "running");
  const closeStory = useStoryStore((s) => s.closeStory);
  const { mode, toggleMode } = useThemeActions();
  return (
    <nav className="flex h-full w-14 shrink-0 flex-col items-center gap-1 border-r bg-card py-3">
      <div className="mb-3 flex size-9 items-center justify-center rounded-lg bg-primary font-semibold text-primary-foreground">Q</div>
      {ITEMS.map(({ page: item, label, icon: Icon }) => (
        <RailButton key={item} label={label} active={page === item} onClick={() => setPage(item)}>
          <Icon className="size-5" />
        </RailButton>
      ))}
      <div className="flex-1" />
      <RailButton label={mode === "dark" ? "Chuyển sang sáng" : "Chuyển sang tối"} onClick={() => void toggleMode()}>
        {mode === "dark" ? <Sun className="size-5" /> : <Moon className="size-5" />}
      </RailButton>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button variant="ghost" size="icon" aria-label="Về danh sách truyện" disabled={running} onClick={closeStory} className="size-10 rounded-lg text-muted-foreground">
            <LibraryBig className="size-5" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">{running ? "Dừng phiên trước khi đổi truyện" : "Về danh sách truyện"}</TooltipContent>
      </Tooltip>
    </nav>
  );
}
```

(`import type React from "react"` nếu cần cho `React.ReactNode`, hoặc dùng `import { type ReactNode } from "react"`.)

- [ ] **Step 7: `src/components/agy-missing.tsx` (restyle)**

```tsx
import { ExternalLink, FolderSearch, RefreshCw, TerminalSquare } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { AgyStatus } from "@/lib/types";

interface Props {
  status: AgyStatus;
  onRetry: () => void;
  onPickPath: () => void;
}

export function AgyMissing({ status, onRetry, onPickPath }: Props) {
  return (
    <main className="flex h-full items-center justify-center p-8">
      <section className="w-full max-w-xl rounded-xl border bg-card p-8 shadow-sm">
        <div className="mb-4 flex size-12 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <TerminalSquare className="size-6" />
        </div>
        <h1 className="text-2xl font-semibold tracking-tight">Chưa thấy Antigravity CLI</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          App dịch bằng quota Antigravity của bạn qua lệnh <code className="font-mono">agy</code>. Cài một lần, đăng nhập Google, rồi quay lại đây.
        </p>
        <ol className="mt-6 space-y-4 text-sm">
          {[
            <>Mở PowerShell, chạy:<pre className="mt-2 rounded-md bg-muted p-3 font-mono text-xs">irm https://antigravity.google/cli/install.ps1 | iex</pre></>,
            <>Mở terminal mới, gõ <code className="font-mono">agy</code> và đăng nhập Google theo hướng dẫn.</>,
            <>Bấm <strong>Kiểm tra lại</strong>.</>,
          ].map((content, index) => (
            <li key={index} className="flex gap-3">
              <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-secondary text-xs font-semibold text-secondary-foreground">{index + 1}</span>
              <div className="min-w-0 flex-1">{content}</div>
            </li>
          ))}
        </ol>
        {status.message && <p className="mt-4 rounded-md border border-destructive/30 bg-destructive/10 p-3 text-xs text-destructive">{status.message}</p>}
        <div className="mt-6 flex flex-wrap gap-2">
          <Button onClick={onRetry}><RefreshCw /> Kiểm tra lại</Button>
          <Button variant="outline" onClick={onPickPath}><FolderSearch /> Chọn file agy tay</Button>
          <Button variant="ghost" asChild>
            <a href="https://antigravity.google/docs/cli/install" target="_blank" rel="noreferrer"><ExternalLink /> Hướng dẫn</a>
          </Button>
        </div>
      </section>
    </main>
  );
}
```

- [ ] **Step 8: `src/components/story-picker.tsx` (restyle + tóm tắt gần đây)**

```tsx
import { FolderOpen, Sparkles } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { ApiError, initStory, openStory, pickFolder, recentSummaries } from "@/lib/api";
import type { RecentSummary } from "@/lib/types";
import { useStoryStore } from "@/store/story";

export function StoryPicker() {
  const recent = useStoryStore((s) => s.config?.recent ?? []);
  const open = useStoryStore((s) => s.openStory);
  const [summaries, setSummaries] = useState<RecentSummary[]>([]);
  const [pendingInit, setPendingInit] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (recent.length === 0) return;
    recentSummaries()
      .then((list) => {
        if (!cancelled) setSummaries(list);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [recent]);

  async function tryOpen(root: string) {
    setBusy(true);
    try {
      open(await openStory(root));
    } catch (error) {
      if (error instanceof ApiError && error.kind === "story_not_found") setPendingInit(root);
      else toast.error(error instanceof Error ? error.message : "Không mở được truyện");
    } finally {
      setBusy(false);
    }
  }

  async function confirmInit() {
    if (!pendingInit) return;
    setBusy(true);
    try {
      open(await initStory(pendingInit));
      toast.success("Đã khởi tạo folder truyện");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không khởi tạo được");
    } finally {
      setBusy(false);
      setPendingInit(undefined);
    }
  }

  async function pickAndOpen() {
    const root = await pickFolder("Chọn folder truyện");
    if (root) await tryOpen(root);
  }

  const rows: RecentSummary[] = recent.map((root) => summaries.find((s) => s.root === root) ?? { root, name: null, done: null, total: null });

  return (
    <main className="flex h-full items-start justify-center overflow-auto p-8">
      <div className="w-full max-w-2xl">
        <header className="mb-8">
          <p className="text-xs font-medium tracking-widest text-primary uppercase">QT AI Translator</p>
          <h1 className="mt-1 text-3xl font-semibold tracking-tight">Chọn truyện để dịch</h1>
          <p className="mt-2 text-sm text-muted-foreground">Folder truyện có thư mục <code className="font-mono">raw/</code> chứa các chương <code className="font-mono">.txt</code>. Folder mới sẽ được khởi tạo.</p>
        </header>
        <Button size="lg" disabled={busy} onClick={() => void pickAndOpen()}><FolderOpen /> Mở folder truyện</Button>
        {rows.length > 0 && (
          <section className="mt-8">
            <h2 className="mb-2 text-xs font-medium tracking-widest text-muted-foreground uppercase">Mở gần đây</h2>
            <ul className="grid gap-2">
              {rows.map((item) => {
                const percent = item.total ? Math.round(((item.done ?? 0) / item.total) * 100) : 0;
                const broken = item.total === null;
                return (
                  <li key={item.root}>
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => void tryOpen(item.root)}
                      className="flex w-full items-center gap-4 rounded-lg border bg-card p-4 text-left transition-colors hover:bg-accent disabled:opacity-50"
                    >
                      <div className="min-w-0 flex-1">
                        <p className={broken ? "truncate text-sm text-muted-foreground" : "truncate font-medium"}>{item.name ?? item.root}</p>
                        <p className="truncate font-mono text-xs text-muted-foreground">{item.root}</p>
                      </div>
                      {!broken && (
                        <div className="w-32 shrink-0 text-right">
                          <p className="text-sm tabular-nums">{item.done}/{item.total}</p>
                          <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-muted"><div className="h-full bg-status-done" style={{ width: `${percent}%` }} /></div>
                        </div>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          </section>
        )}
      </div>
      <Dialog open={pendingInit !== undefined} onOpenChange={(value) => { if (!value) setPendingInit(undefined); }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Khởi tạo folder truyện?</DialogTitle>
            <DialogDescription>
              <code className="font-mono">{pendingInit}</code> chưa có <code>state.json</code>. Khởi tạo sẽ tạo <code>story.json</code>, <code>state.json</code>, <code>AGENTS.md</code> và đưa mọi chương trong <code>raw/</code> vào hàng đợi. Không đụng file gốc.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPendingInit(undefined)}>Bỏ</Button>
            <Button disabled={busy} onClick={() => void confirmInit()}><Sparkles /> Khởi tạo</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </main>
  );
}
```

- [ ] **Step 9: `src/app.tsx`**

```tsx
import { LoaderCircle } from "lucide-react";
import { useCallback, useEffect } from "react";
import { toast } from "sonner";

import { AgyMissing } from "@/components/agy-missing";
import { AppRail } from "@/components/app-rail";
import { StoryPicker } from "@/components/story-picker";
import { useSessionEvents } from "@/hooks/use-session-events";
import { useThemeSync } from "@/hooks/use-theme";
import { agyStatus, appConfigGet, appConfigSet, pickAgyFile } from "@/lib/api";
import { useStoryStore, type Page } from "@/store/story";

// Task 4–6 thay bằng import trang thật
const PAGES: Record<Page, () => React.ReactElement> = {
  translate: () => <div className="p-6 text-sm">Dịch</div>,
  story: () => <div className="p-6 text-sm">Hồ sơ</div>,
  export: () => <div className="p-6 text-sm">Export</div>,
  settings: () => <div className="p-6 text-sm">Cài đặt</div>,
};

export default function App() {
  const agy = useStoryStore((s) => s.agy);
  const screen = useStoryStore((s) => s.screen);
  const page = useStoryStore((s) => s.page);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setConfig = useStoryStore((s) => s.setConfig);
  useSessionEvents();
  useThemeSync();

  const probe = useCallback(async () => {
    try {
      const config = await appConfigGet();
      setConfig(config);
      setAgy(await agyStatus(config.agyPath ?? undefined));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đọc được cấu hình");
    }
  }, [setAgy, setConfig]);

  useEffect(() => {
    void probe();
  }, [probe]);

  async function pickAgy() {
    const path = await pickAgyFile();
    if (!path) return;
    const config = useStoryStore.getState().config;
    if (!config) return;
    setConfig(await appConfigSet({ ...config, agyPath: path }));
    await probe();
  }

  if (!agy) {
    return (
      <main className="flex h-full items-center justify-center text-sm text-muted-foreground">
        <LoaderCircle className="mr-2 animate-spin" /> Đang kiểm tra agy…
      </main>
    );
  }
  if (!agy.found) return <AgyMissing status={agy} onRetry={() => void probe()} onPickPath={() => void pickAgy()} />;
  if (screen === "picker") return <StoryPicker />;
  const Current = PAGES[page];
  return (
    <div className="flex h-full">
      <AppRail />
      <main className="min-w-0 flex-1 overflow-hidden">
        <Current />
      </main>
    </div>
  );
}
```

(`import type React from "react"` cho `React.ReactElement`, hoặc `type ReactElement`.) `workbench.tsx` tạm không còn được import — chưa xoá (Task 7).

- [ ] **Step 10: Kiểm**

Run: `npx vitest run && npm run -s typecheck && npm run -s lint` — xanh. `npm run tauri dev`: màn chọn truyện mới hiện tên + tiến độ (folder hỏng mờ), mở truyện → rail + trang placeholder, nút mặt trăng đổi tối/sáng, đổi xong tắt app mở lại vẫn giữ.

- [ ] **Step 11: Commit**

```bash
git add apps/qt-ai-gui/src
git commit -m "feat(qt-ai-gui): store page + bộ lọc chương, hook theme đồng bộ AppConfig, rail trái, màn agy/chọn truyện mới

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 4: Trang Dịch — toolbar, danh sách chương (chip + tìm), reader, log

**Files:**
- Create: `src/components/pages/translate-page.tsx`, `src/components/translate-toolbar.tsx`, `src/components/chapter-list.tsx`, `src/components/chapter-list.test.tsx`, `src/components/chapter-reader.tsx`
- Modify: `src/components/log-panel.tsx`, `src/app.tsx` (PAGES.translate)

**Interfaces:**
- `<ChapterList rows filter query selectedId onSelect onFilter onQuery />`; `<TranslateToolbar />`; `<ChapterReader root row onPrev onNext hasPrev hasNext />`; `<LogPanel />`; `<TranslatePage />`.
- Consumes: `filterChapters/countByFilter/FILTER_ORDER/FILTER_LABELS` (Task 3), store, api.

- [ ] **Step 1: Test `src/components/chapter-list.test.tsx`**

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ChapterList } from "@/components/chapter-list";
import type { ChapterRow } from "@/lib/types";

const rows: ChapterRow[] = [
  { id: "0001", status: "done", reviewRound: 0, reason: null, warnings: [] },
  { id: "0002", status: "done", reviewRound: 3, reason: null, warnings: ["[[1]] CJK"] },
  { id: "0003", status: "error", reviewRound: 3, reason: "Quá 3 vòng", warnings: [] },
  { id: "0004", status: "queued", reviewRound: 0, reason: null, warnings: [] },
];

describe("ChapterList", () => {
  it("hiện chip đếm, lọc theo prop, gọi onSelect/onFilter/onQuery", async () => {
    const onSelect = vi.fn();
    const onFilter = vi.fn();
    const onQuery = vi.fn();
    render(<ChapterList rows={rows} filter="all" query="" selectedId={undefined} onSelect={onSelect} onFilter={onFilter} onQuery={onQuery} />);
    expect(screen.getByRole("button", { name: /Lỗi 1/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Cảnh báo 1/ })).toBeInTheDocument();
    expect(screen.getAllByRole("option")).toHaveLength(4);
    await userEvent.click(screen.getByRole("button", { name: /Lỗi 1/ }));
    expect(onFilter).toHaveBeenCalledWith("error");
    await userEvent.click(screen.getByRole("option", { name: /0003/ }));
    expect(onSelect).toHaveBeenCalledWith("0003");
    await userEvent.type(screen.getByRole("searchbox"), "2");
    expect(onQuery).toHaveBeenLastCalledWith("2");
  });

  it("lọc error chỉ còn 1 hàng, rỗng thì báo", () => {
    const { rerender } = render(<ChapterList rows={rows} filter="error" query="" selectedId="0003" onSelect={() => undefined} onFilter={() => undefined} onQuery={() => undefined} />);
    expect(screen.getAllByRole("option")).toHaveLength(1);
    expect(screen.getByRole("option", { name: /0003/ })).toHaveAttribute("aria-selected", "true");
    rerender(<ChapterList rows={rows} filter="skipped" query="" selectedId={undefined} onSelect={() => undefined} onFilter={() => undefined} onQuery={() => undefined} />);
    expect(screen.getByText("Không có chương nào khớp.")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Chạy để thấy fail** — `npx vitest run src/components/chapter-list.test.tsx` → FAIL.

- [ ] **Step 3: `src/components/chapter-list.tsx`**

```tsx
import { Search } from "lucide-react";

import { Input } from "@/components/ui/input";
import { countByFilter, FILTER_LABELS, FILTER_ORDER, filterChapters, type ChapterFilter } from "@/lib/chapters";
import type { ChapterRow, ChapterStatus } from "@/lib/types";
import { cn } from "@/lib/utils";

const DOT: Record<ChapterStatus, string> = {
  queued: "bg-status-queued",
  translating: "bg-status-translating animate-pulse",
  done: "bg-status-done",
  error: "bg-status-error",
  skipped: "bg-status-queued ring-1 ring-foreground/30",
};

interface Props {
  rows: ChapterRow[];
  filter: ChapterFilter;
  query: string;
  selectedId?: string;
  onSelect: (id: string) => void;
  onFilter: (filter: ChapterFilter) => void;
  onQuery: (query: string) => void;
}

export function ChapterList({ rows, filter, query, selectedId, onSelect, onFilter, onQuery }: Props) {
  const visible = filterChapters(rows, filter, query);
  const counts = countByFilter(rows);
  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-2 border-b p-3">
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input type="search" role="searchbox" value={query} onChange={(e) => onQuery(e.target.value)} placeholder="Tìm mã chương…" className="h-8 pl-8 font-mono text-xs" />
        </div>
        <div className="flex flex-wrap gap-1">
          {FILTER_ORDER.filter((f) => f === "all" || counts[f] > 0).map((f) => (
            <button
              key={f}
              type="button"
              onClick={() => onFilter(f)}
              aria-pressed={filter === f}
              className={cn(
                "rounded-full border px-2 py-0.5 text-xs transition-colors hover:bg-accent",
                filter === f ? "border-primary bg-primary text-primary-foreground hover:bg-primary" : "text-muted-foreground",
              )}
            >
              {FILTER_LABELS[f]} {counts[f]}
            </button>
          ))}
        </div>
      </div>
      <ul role="listbox" aria-label="Danh sách chương" className="fine-scrollbar flex-1 overflow-y-auto">
        {visible.length === 0 && <li className="p-4 text-sm text-muted-foreground">Không có chương nào khớp.</li>}
        {visible.map((row) => {
          const selected = selectedId === row.id;
          return (
            <li
              key={row.id}
              role="option"
              aria-selected={selected}
              tabIndex={0}
              onClick={() => onSelect(row.id)}
              onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onSelect(row.id); } }}
              className={cn(
                "flex cursor-pointer items-center gap-2.5 border-l-2 border-transparent px-3 py-2 text-sm hover:bg-accent/60",
                selected && "border-l-primary bg-accent",
              )}
            >
              <span className={cn("size-2 shrink-0 rounded-full", DOT[row.status])} aria-hidden />
              <span className="min-w-0 flex-1 truncate font-mono text-xs">{row.id}</span>
              {row.reviewRound > 0 && <span className="text-[11px] text-muted-foreground">soát {row.reviewRound}</span>}
              {row.warnings.length > 0 && (
                <span className="rounded-full bg-status-warning/15 px-1.5 text-[11px] font-medium text-status-warning">{row.warnings.length}</span>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
```

- [ ] **Step 4: `src/components/translate-toolbar.tsx`**

```tsx
import { Play, Square } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { sessionStart, sessionStop, storySnapshot } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

function Stat({ label, value, tone }: { label: string; value: number; tone: string }) {
  return (
    <span className={cn("inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-xs", tone)}>
      <span className="size-1.5 rounded-full bg-current" aria-hidden />
      {label} <span className="font-medium tabular-nums">{value}</span>
    </span>
  );
}

export function TranslateToolbar() {
  const root = useStoryStore((s) => s.root);
  const snapshot = useStoryStore((s) => s.snapshot);
  const session = useStoryStore((s) => s.session);
  const progress = useStoryStore((s) => s.progress);
  const agy = useStoryStore((s) => s.agy);
  const config = useStoryStore((s) => s.config);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const [model, setModel] = useState<string | undefined>(config?.model ?? undefined);
  const [busy, setBusy] = useState(false);

  const counts = snapshot?.counts;
  const done = progress?.done ?? counts?.done ?? 0;
  const total = counts?.total ?? 0;
  const percent = total > 0 ? Math.round((done / total) * 100) : 0;
  const running = session.status === "running";

  async function toggle() {
    if (!root) return;
    setBusy(true);
    try {
      if (running) {
        await sessionStop();
        setSnapshot(await storySnapshot(root));
      } else {
        await sessionStart(root, model);
        toast.message("Đã bắt đầu phiên dịch");
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đổi được trạng thái phiên");
    } finally {
      setBusy(false);
    }
  }

  return (
    <header className={cn("border-b bg-card px-5 py-3 transition-colors", running && "border-b-primary/50")}>
      <div className="flex items-center gap-3">
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-base font-semibold tracking-tight">{snapshot?.story.name || root}</h1>
          <p className="truncate font-mono text-xs text-muted-foreground">{root}</p>
        </div>
        <Select value={model ?? ""} onValueChange={(value) => setModel(value || undefined)} disabled={running}>
          <SelectTrigger className="h-9 w-56" aria-label="Model"><SelectValue placeholder="Model mặc định của agy" /></SelectTrigger>
          <SelectContent>{(agy?.models ?? []).map((item) => <SelectItem key={item} value={item}>{item}</SelectItem>)}</SelectContent>
        </Select>
        <Button size="lg" variant={running ? "destructive" : "default"} disabled={busy} onClick={() => void toggle()} className="min-w-36">
          {running ? <><Square /> Dừng</> : <><Play /> Bắt đầu dịch</>}
        </Button>
      </div>
      <div className="mt-3 flex items-center gap-3">
        <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
          <div className="h-full rounded-full bg-status-done transition-[width]" style={{ width: `${percent}%` }} />
        </div>
        <span className="text-sm tabular-nums">{done}/{total} <span className="text-muted-foreground">({percent}%)</span></span>
      </div>
      <div className="mt-2 flex flex-wrap items-center gap-1.5">
        {counts && (
          <>
            <Stat label="Chờ" value={progress?.queued ?? counts.queued} tone="text-muted-foreground" />
            <Stat label="Lỗi" value={progress?.error ?? counts.error} tone="text-status-error border-status-error/30" />
            <Stat label="Bỏ qua" value={progress?.skipped ?? counts.skipped} tone="text-muted-foreground" />
            <Stat label="Cảnh báo" value={progress?.warnings_count ?? counts.withWarnings} tone="text-status-warning border-status-warning/30" />
          </>
        )}
        {running && (
          <span className="ml-auto inline-flex items-center gap-2 text-xs text-primary">
            <span className="size-2 animate-pulse rounded-full bg-status-translating" aria-hidden />
            Phiên {session.sessionNo}{progress?.current ? ` · đang dịch ${progress.current}` : ""}
          </span>
        )}
      </div>
    </header>
  );
}
```

- [ ] **Step 5: `src/components/chapter-reader.tsx`**

```tsx
import { AlertTriangle, ChevronLeft, ChevronRight, FolderOpen, RotateCcw, ShieldCheck, SkipForward } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { LogPanel } from "@/components/log-panel";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { chapterForceAccept, chapterRetry, chapterSkip, readChapter, revealFolder, storySnapshot } from "@/lib/api";
import { STATUS_LABELS, type ChapterRow, type ChapterStatus, type ChapterView } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

const STATUS_TONE: Record<ChapterStatus, string> = {
  queued: "bg-status-queued/30 text-foreground",
  translating: "bg-status-translating/15 text-status-translating",
  done: "bg-status-done/15 text-status-done",
  error: "bg-status-error/15 text-status-error",
  skipped: "bg-muted text-muted-foreground",
};

interface Props {
  root: string;
  row: ChapterRow;
  hasPrev: boolean;
  hasNext: boolean;
  onPrev: () => void;
  onNext: () => void;
}

export function ChapterReader({ root, row, hasPrev, hasNext, onPrev, onNext }: Props) {
  const running = useStoryStore((s) => s.session.status === "running");
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const viewKey = `${root}|${row.id}|${row.status}|${row.reviewRound}`;
  const [loaded, setLoaded] = useState<{ key: string; view: ChapterView } | undefined>();
  const view = loaded?.key === viewKey ? loaded.view : undefined;
  const [skipOpen, setSkipOpen] = useState(false);
  const [reason, setReason] = useState("");

  useEffect(() => {
    let cancelled = false;
    readChapter(root, row.id)
      .then((v) => { if (!cancelled) setLoaded({ key: viewKey, view: v }); })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : "Không đọc được chương"));
    return () => { cancelled = true; };
  }, [root, row.id, viewKey]);

  async function act(label: string, action: () => Promise<unknown>) {
    try {
      await action();
      setSnapshot(await storySnapshot(root));
      toast.success(label);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : `${label} thất bại`);
    }
  }

  const canRetry = row.status === "error" || row.status === "skipped";
  const canSkip = row.status !== "done";
  const canForce = row.status === "translating" && Boolean(view?.draft);
  const defaultTab = view?.output ? "output" : view?.draft ? "draft" : "raw";

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-2 border-b px-5 py-3">
        <span className="font-mono text-sm font-medium">{row.id}</span>
        <span className={cn("rounded-full px-2 py-0.5 text-xs font-medium", STATUS_TONE[row.status])}>{STATUS_LABELS[row.status]}</span>
        {row.reason && <span className="truncate text-xs text-muted-foreground">{row.reason}</span>}
        <div className="flex-1" />
        <Button size="sm" variant="outline" disabled={!canRetry || running} onClick={() => void act("Đã đưa về hàng đợi", () => chapterRetry(root, row.id))}><RotateCcw /> Dịch lại</Button>
        <Button size="sm" variant="outline" disabled={!canSkip || running} onClick={() => setSkipOpen(true)}><SkipForward /> Bỏ qua</Button>
        <Button size="sm" variant="outline" disabled={!canForce || running} title="Chốt bản nháp hiện có dù check chưa đạt" onClick={() => void act("Đã chốt (force)", () => chapterForceAccept(root, row.id))}><ShieldCheck /> Chốt --force</Button>
        <Button size="sm" variant="ghost" onClick={() => void revealFolder(root)}><FolderOpen /> Mở folder</Button>
      </div>
      {row.warnings.length > 0 && (
        <div className="border-b bg-status-warning/10 px-5 py-2 text-xs">
          <div className="mb-1 flex items-center gap-1.5 font-medium text-status-warning"><AlertTriangle className="size-3.5" /> {row.warnings.length} cảnh báo còn lại</div>
          <ul className="list-disc space-y-0.5 pl-5 text-foreground/80">{row.warnings.map((w) => <li key={w}>{w}</li>)}</ul>
        </div>
      )}
      <Tabs key={viewKey} defaultValue={defaultTab} className="flex min-h-0 flex-1 flex-col gap-0">
        <TabsList variant="line" className="mx-5 mt-2 border-b">
          <TabsTrigger value="output" disabled={!view?.output}>Bản dịch</TabsTrigger>
          <TabsTrigger value="draft" disabled={!view?.draft}>Nháp</TabsTrigger>
          <TabsTrigger value="review" disabled={!view?.review}>Yêu cầu sửa</TabsTrigger>
          <TabsTrigger value="raw">Gốc</TabsTrigger>
          <TabsTrigger value="log">Log</TabsTrigger>
        </TabsList>
        {(["output", "draft", "review", "raw"] as const).map((key) => (
          <TabsContent key={key} value={key} className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
            <article className="reading mx-auto px-6 py-8">
              <pre className="font-[inherit] whitespace-pre-wrap">{view ? (key === "raw" ? view.raw : (view[key] ?? "")) : "Đang đọc…"}</pre>
              <nav className="mt-10 flex items-center justify-between border-t pt-4 font-sans text-sm">
                <Button variant="ghost" size="sm" disabled={!hasPrev} onClick={onPrev}><ChevronLeft /> Chương trước</Button>
                <Button variant="ghost" size="sm" disabled={!hasNext} onClick={onNext}>Chương sau <ChevronRight /></Button>
              </nav>
            </article>
          </TabsContent>
        ))}
        <TabsContent value="log" className="min-h-0 flex-1"><LogPanel /></TabsContent>
      </Tabs>
      <Dialog open={skipOpen} onOpenChange={setSkipOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Bỏ qua chương {row.id}</DialogTitle></DialogHeader>
          <Input value={reason} onChange={(e) => setReason(e.target.value)} placeholder="Lý do (vd. model từ chối nội dung)" />
          <DialogFooter>
            <Button variant="outline" onClick={() => setSkipOpen(false)}>Huỷ</Button>
            <Button disabled={!reason.trim()} onClick={() => { setSkipOpen(false); void act("Đã bỏ qua", () => chapterSkip(root, row.id, reason)); setReason(""); }}>Bỏ qua</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
```

Kiểm `TabsList` có prop `variant="line"` (đã có `tabsListVariants` với `line` trong `ui/tabs.tsx`; nếu component không nhận prop `variant`, thêm `variant` vào props của `TabsList` theo pattern cva sẵn có).

- [ ] **Step 6: `src/components/log-panel.tsx` (token hoá)**

```tsx
import { Eraser } from "lucide-react";
import { useEffect, useRef } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

export function LogPanel() {
  const logs = useStoryStore((s) => s.logs);
  const clear = useStoryStore((s) => s.clearLogs);
  const bottom = useRef<HTMLDivElement>(null);
  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [logs.length]);
  return (
    <div className="flex h-full flex-col bg-log text-log-foreground">
      <div className="flex items-center justify-between border-b border-log-foreground/10 px-3 py-1.5">
        <span className="font-mono text-xs opacity-70">{logs.length} dòng log agy</span>
        <Button size="xs" variant="ghost" onClick={clear} className="text-log-foreground hover:bg-log-foreground/10 hover:text-log-foreground"><Eraser /> Xoá</Button>
      </div>
      <div className="fine-scrollbar flex-1 overflow-auto p-3 font-mono text-xs leading-relaxed">
        {logs.length === 0 && <p className="opacity-60">Chưa có log. Bấm Bắt đầu dịch để agy chạy.</p>}
        {logs.map((entry) => (
          <div key={entry.seq} className={cn("whitespace-pre-wrap", entry.stream === "stderr" && "text-status-warning")}>{entry.line}</div>
        ))}
        <div ref={bottom} />
      </div>
    </div>
  );
}
```

- [ ] **Step 7: `src/components/pages/translate-page.tsx`**

```tsx
import { ChapterList } from "@/components/chapter-list";
import { ChapterReader } from "@/components/chapter-reader";
import { TranslateToolbar } from "@/components/translate-toolbar";
import { filterChapters } from "@/lib/chapters";
import { useStoryStore } from "@/store/story";

export function TranslatePage() {
  const root = useStoryStore((s) => s.root);
  const snapshot = useStoryStore((s) => s.snapshot);
  const selectedId = useStoryStore((s) => s.selectedId);
  const filter = useStoryStore((s) => s.statusFilter);
  const query = useStoryStore((s) => s.searchQuery);
  const select = useStoryStore((s) => s.select);
  const setFilter = useStoryStore((s) => s.setStatusFilter);
  const setQuery = useStoryStore((s) => s.setSearchQuery);
  if (!root || !snapshot) return null;
  const visible = filterChapters(snapshot.chapters, filter, query);
  const index = visible.findIndex((c) => c.id === selectedId);
  const row = index >= 0 ? visible[index] : snapshot.chapters.find((c) => c.id === selectedId);
  return (
    <div className="flex h-full flex-col">
      <TranslateToolbar />
      <div className="grid min-h-0 flex-1 grid-cols-[280px_1fr]">
        <aside className="min-h-0 border-r bg-card/50">
          <ChapterList rows={snapshot.chapters} filter={filter} query={query} selectedId={selectedId} onSelect={select} onFilter={setFilter} onQuery={setQuery} />
        </aside>
        <section className="min-h-0 min-w-0">
          {row ? (
            <ChapterReader
              root={root}
              row={row}
              hasPrev={index > 0}
              hasNext={index >= 0 && index < visible.length - 1}
              onPrev={() => { const prev = visible[index - 1]; if (prev) select(prev.id); }}
              onNext={() => { const next = visible[index + 1]; if (next) select(next.id); }}
            />
          ) : (
            <div className="flex h-full flex-col items-center justify-center gap-2 text-center text-sm text-muted-foreground">
              <p className="text-base font-medium text-foreground">Chọn một chương bên trái</p>
              <p>Bản dịch, bản nháp, bản gốc và log agy sẽ hiện ở đây.</p>
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
```

`app.tsx`: `import { TranslatePage } from "@/components/pages/translate-page";` và `translate: TranslatePage,` trong `PAGES`.

- [ ] **Step 8: Kiểm**

Run: `npx vitest run && npm run -s typecheck && npm run -s lint` — chapter-list 2 pass, tổng xanh, 0 error. `tauri dev`: chip đếm đúng, tìm lọc realtime, chọn chương đọc được ở cột 70ch chữ serif (Editorial) / sans (Studio, Soft), chương trước/sau đi trong danh sách đã lọc, tab Log nền tối ở cả hai mode.

- [ ] **Step 9: Commit**

```bash
git add apps/qt-ai-gui/src
git commit -m "feat(qt-ai-gui): trang Dịch mới — toolbar tiến độ, danh sách chương chip+tìm, reader 70ch có prev/next, log token hoá

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 5: Trang Hồ sơ truyện (form một trang, mục lục neo, glossary tìm nhanh)

**Files:**
- Create: `src/components/pages/story-page.tsx`
- Modify: `src/components/glossary-editor.tsx`, `src/components/ai-fill-dialog.tsx`, `src/app.tsx`

**Interfaces:**
- `<StoryPage />`; `<GlossaryEditor name label />` (thêm ô tìm); AiFillDialog giữ props cũ.
- Consumes: `story-form.ts` (`toFormValues/fromFormValues/storyFormSchema`), `CheckRulesEditor`, api `saveStory/storySnapshot`.

- [ ] **Step 1: `src/components/glossary-editor.tsx`**

```tsx
import { Plus, Search, Trash2 } from "lucide-react";
import { useState } from "react";
import { useFieldArray, useFormContext, useWatch } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { StoryFormValues } from "@/lib/story-form";

interface Props {
  name: `glossary.${keyof StoryFormValues["glossary"]}` | "signaturePhrases";
  label: string;
}

/** Bảng CN → VN thêm/xoá dòng, có ô tìm nhanh lọc theo cả hai cột (index giữ nguyên để register đúng dòng). */
export function GlossaryEditor({ name, label }: Props) {
  const { control, register } = useFormContext<StoryFormValues>();
  const { fields, append, remove } = useFieldArray({ control, name });
  const values = useWatch({ control, name }) ?? [];
  const [query, setQuery] = useState("");
  const needle = query.trim().toLowerCase();
  const visible = fields
    .map((field, index) => ({ field, index }))
    .filter(({ index }) => {
      if (!needle) return true;
      const pair = values[index];
      return Boolean(pair && (pair.source.toLowerCase().includes(needle) || pair.target.toLowerCase().includes(needle)));
    });
  return (
    <fieldset className="rounded-lg border bg-card p-4">
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <legend className="text-sm font-semibold">{label} <span className="ml-1 rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">{fields.length}</span></legend>
        <div className="flex-1" />
        {fields.length > 8 && (
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Tìm…" aria-label={`Tìm trong ${label}`} className="h-7 w-40 pl-7 text-xs" />
          </div>
        )}
        <Button type="button" size="xs" variant="secondary" onClick={() => { setQuery(""); append({ source: "", target: "" }); }}><Plus /> Thêm</Button>
      </div>
      <div className="flex flex-col gap-1">
        {visible.length === 0 && <p className="py-2 text-xs text-muted-foreground">{fields.length === 0 ? "Chưa có mục nào." : "Không khớp."}</p>}
        {visible.map(({ field, index }) => (
          <div key={field.id} className="grid grid-cols-[1fr_1fr_auto] gap-1">
            <Input {...register(`${name}.${index}.source`)} placeholder="Hán tự" aria-label={`${label} CN ${index + 1}`} className="h-8 font-mono" />
            <Input {...register(`${name}.${index}.target`)} placeholder="Tiếng Việt" aria-label={`${label} VN ${index + 1}`} className="h-8" />
            <Button type="button" size="icon-sm" variant="ghost" aria-label="Xoá dòng" onClick={() => remove(index)}><Trash2 /></Button>
          </div>
        ))}
      </div>
    </fieldset>
  );
}
```

- [ ] **Step 2: `src/components/ai-fill-dialog.tsx` — token hoá phần log**

Thay `className="border-t bg-zinc-950 p-2 font-mono text-[11px] text-zinc-200"` bằng `className="border-t bg-log p-2 font-mono text-[11px] text-log-foreground"`. Thay chữ "Không ghi gì cho tới khi bạn bấm Áp dụng." giữ nguyên (đã không có mày/tao).

- [ ] **Step 3: `src/components/pages/story-page.tsx`**

```tsx
import { zodResolver } from "@hookform/resolvers/zod";
import { Download, Save, Sparkles, Upload } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import { AiFillDialog } from "@/components/ai-fill-dialog";
import { CheckRulesEditor } from "@/components/check-rules-editor";
import { GlossaryEditor } from "@/components/glossary-editor";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { saveStory, storySnapshot } from "@/lib/api";
import { storyConfigSchema } from "@/lib/schema";
import { fromFormValues, storyFormSchema, toFormValues, type StoryFormValues } from "@/lib/story-form";
import { GLOSSARY_KEYS, GLOSSARY_LABELS, type StoryConfig } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

const SECTIONS = [
  { id: "info", label: "Thông tin" },
  { id: "style", label: "Style" },
  { id: "glossary", label: "Glossary" },
  { id: "rules", label: "Rule kiểm tra" },
  { id: "prompt", label: "Prompt" },
] as const;

function Field({ id, label, hint, children }: { id: string; label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={id}>{label}</Label>
      {children}
      {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
    </div>
  );
}

function Section({ id, title, children }: { id: string; title: string; children: React.ReactNode }) {
  return (
    <section id={`sec-${id}`} className="scroll-mt-4">
      <h2 className="mb-3 text-lg font-semibold tracking-tight">{title}</h2>
      <div className="flex flex-col gap-4">{children}</div>
    </section>
  );
}

export function StoryPage() {
  const root = useStoryStore((s) => s.root);
  const story = useStoryStore((s) => s.snapshot?.story);
  const running = useStoryStore((s) => s.session.status === "running");
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const form = useForm<StoryFormValues>({ resolver: zodResolver(storyFormSchema), defaultValues: story ? toFormValues(story) : undefined });
  const autoGlossary = useWatch({ control: form.control, name: "autoGlossary" });
  const [fillOpen, setFillOpen] = useState(false);
  const [active, setActive] = useState<string>("info");
  const fileInput = useRef<HTMLInputElement>(null);
  const scroller = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (story) form.reset(toFormValues(story));
  }, [story, form]);

  if (!root || !story) return null;
  const currentRoot = root;
  const currentStory = story;

  async function persist(config: StoryConfig) {
    await saveStory(currentRoot, config);
    setSnapshot(await storySnapshot(currentRoot));
    toast.success("Đã lưu hồ sơ truyện");
  }

  const submit = form.handleSubmit(async (values) => {
    try {
      await persist(fromFormValues(values, currentStory));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    }
  });

  function exportJson() {
    const config = fromFormValues(form.getValues(), currentStory);
    const blob = new Blob([`${JSON.stringify(config, null, 2)}\n`], { type: "application/json" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = `${config.name || "story"}.story.json`;
    link.click();
    URL.revokeObjectURL(link.href);
  }

  async function importJson(file: File) {
    try {
      const parsed = storyConfigSchema.parse(JSON.parse(await file.text()));
      form.reset(toFormValues({ ...parsed, autoGlossaryLog: currentStory.autoGlossaryLog }), { keepDefaultValues: true });
      toast.message("Đã nạp JSON vào form — bấm Lưu để ghi");
    } catch {
      toast.error("File không đúng schema story.json của qt-web");
    }
  }

  function jumpTo(id: string) {
    setActive(id);
    scroller.current?.querySelector(`#sec-${id}`)?.scrollIntoView({ block: "start", behavior: "smooth" });
  }

  const dirty = form.formState.isDirty;

  return (
    <FormProvider {...form}>
      <form onSubmit={(e) => void submit(e)} className="flex h-full flex-col">
        <div className="grid min-h-0 flex-1 grid-cols-[200px_1fr]">
          <nav className="border-r bg-card/50 p-4">
            <p className="mb-3 text-xs font-medium tracking-widest text-muted-foreground uppercase">Hồ sơ truyện</p>
            <ul className="flex flex-col gap-0.5">
              {SECTIONS.map((section) => (
                <li key={section.id}>
                  <button type="button" onClick={() => jumpTo(section.id)} className={cn("w-full rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent", active === section.id && "bg-accent font-medium")}>{section.label}</button>
                </li>
              ))}
            </ul>
          </nav>
          <div ref={scroller} className="fine-scrollbar min-h-0 overflow-y-auto">
            <div className="mx-auto flex max-w-3xl flex-col gap-10 px-8 py-8">
              <Section id="info" title="Thông tin">
                <div className="grid grid-cols-2 gap-4">
                  <Field id="name" label="Tên truyện"><Input id="name" {...form.register("name")} /></Field>
                  <Field id="sourceUrl" label="Link nguồn"><Input id="sourceUrl" {...form.register("sourceUrl")} placeholder="https://…" /></Field>
                  <Field id="protagonist" label="Nhân vật chính" hint="Tên Hán-Việt dùng xuyên suốt."><Input id="protagonist" {...form.register("protagonist")} /></Field>
                  <Field id="autoGlossary" label="Tự thêm tên riêng">
                    <Select value={autoGlossary} onValueChange={(v) => form.setValue("autoGlossary", v as StoryFormValues["autoGlossary"], { shouldDirty: true })}>
                      <SelectTrigger id="autoGlossary"><SelectValue /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="inherit">Theo mặc định (bật)</SelectItem>
                        <SelectItem value="on">Bật</SelectItem>
                        <SelectItem value="off">Tắt</SelectItem>
                      </SelectContent>
                    </Select>
                  </Field>
                </div>
                <Field id="summary" label="Tóm tắt" hint="Bối cảnh, tuyến nhân vật, mạch truyện chính — agent đọc để dịch nhất quán."><Textarea id="summary" rows={6} {...form.register("summary")} /></Field>
                <Button type="button" variant="secondary" className="w-fit" disabled={running} onClick={() => setFillOpen(true)}><Sparkles /> AI điền từ tên + link</Button>
              </Section>
              <Section id="style" title="Style">
                <Field id="voice" label="Giọng kể / voice"><Input id="voice" {...form.register("voice")} /></Field>
                <Field id="toneRules" label="Tone rules" hint="Mỗi dòng một rule."><Textarea id="toneRules" rows={5} {...form.register("toneRules")} /></Field>
                <Field id="avoid" label="Cách diễn đạt cần tránh" hint="Mỗi dòng một mục."><Textarea id="avoid" rows={4} {...form.register("avoid")} /></Field>
                <GlossaryEditor name="signaturePhrases" label="Cụm từ đặc trưng (style)" />
              </Section>
              <Section id="glossary" title="Glossary">
                {GLOSSARY_KEYS.map((key) => <GlossaryEditor key={key} name={`glossary.${key}`} label={GLOSSARY_LABELS[key]} />)}
              </Section>
              <Section id="rules" title="Rule kiểm tra"><CheckRulesEditor /></Section>
              <Section id="prompt" title="Prompt">
                <Field id="customPrompt" label="Custom prompt" hint="Trống = prompt mặc định của hệ."><Textarea id="customPrompt" rows={10} className="font-mono text-xs" {...form.register("customPrompt")} /></Field>
              </Section>
            </div>
          </div>
        </div>
        <footer className="flex items-center gap-2 border-t bg-card px-5 py-3">
          <input ref={fileInput} type="file" accept=".json" className="hidden" aria-label="Chọn file story.json" onChange={(e) => { const f = e.target.files?.[0]; if (f) void importJson(f); e.target.value = ""; }} />
          <Button type="button" variant="ghost" size="sm" onClick={() => fileInput.current?.click()}><Upload /> Nhập JSON</Button>
          <Button type="button" variant="ghost" size="sm" onClick={exportJson}><Download /> Xuất JSON</Button>
          <div className="flex-1" />
          {dirty && <span className="text-xs text-muted-foreground">Có thay đổi chưa lưu</span>}
          <Button type="submit" disabled={running || !dirty || form.formState.isSubmitting}><Save /> Lưu</Button>
        </footer>
        <AiFillDialog
          root={currentRoot}
          initialName={form.getValues("name")}
          initialUrl={form.getValues("sourceUrl")}
          open={fillOpen}
          onOpenChange={setFillOpen}
          onApply={(after) => {
            setFillOpen(false);
            void persist(after).then(() => form.reset(toFormValues(after))).catch((e: unknown) => toast.error(e instanceof Error ? e.message : "Không lưu được"));
          }}
        />
      </form>
    </FormProvider>
  );
}
```

(`import { type ReactNode } from "react"` và dùng `ReactNode` thay `React.ReactNode`.) Lưu ý: `form.reset(..., { keepDefaultValues: true })` khi nhập JSON để `isDirty` = true → nút Lưu bật.

`app.tsx`: `story: StoryPage`.

- [ ] **Step 4: Kiểm**

Run: `npx vitest run && npm run -s typecheck && npm run -s lint`. `tauri dev`: trang Hồ sơ cuộn mượt với glossary hàng trăm dòng, ô tìm hiện khi nhóm > 8 dòng, sửa tên → Lưu bật → lưu → toolbar trang Dịch đổi tên; Nhập JSON bật Lưu; AI điền vẫn là dialog.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src
git commit -m "feat(qt-ai-gui): trang Hồ sơ truyện một trang có mục lục neo, glossary tìm nhanh, thanh lưu dính đáy

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 6: Trang Export (xem trước) + Trang Cài đặt (giao diện, app, truyện)

**Files:**
- Create: `src/components/pages/export-page.tsx`, `src/components/pages/settings-page.tsx`, `src/components/palette-picker.tsx`
- Modify: `src/app.tsx`

**Interfaces:**
- `<ExportPage />`, `<SettingsPage />`, `<PalettePicker value onChange />` (value: `Palette`).
- Consumes: `useThemeActions` (Task 3), `PALETTES/THEME_MODES/THEME_MODE_LABELS` (Task 1), api.

- [ ] **Step 1: `src/components/palette-picker.tsx`**

```tsx
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
            className={cn("rounded-lg border p-3 text-left transition-colors hover:bg-accent/50", selected && "border-primary ring-2 ring-primary/30")}
          >
            <div className="mb-2 grid grid-cols-2 gap-1">
              {(["light", "dark"] as const).map((mode) => {
                const [bg, fg, accent] = palette.preview[mode];
                return (
                  <div key={mode} className="flex h-10 items-center gap-1 rounded-md border px-2" style={{ background: bg, color: fg }}>
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
```

- [ ] **Step 2: `src/components/pages/settings-page.tsx`**

```tsx
import { zodResolver } from "@hookform/resolvers/zod";
import { FolderSearch } from "lucide-react";
import { useEffect, type ComponentProps, type ReactNode } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { PalettePicker } from "@/components/palette-picker";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useThemeActions } from "@/hooks/use-theme";
import { agyStatus, appConfigSet, pickAgyFile, saveSettings, storySnapshot } from "@/lib/api";
import { THEME_MODE_LABELS, THEME_MODES } from "@/lib/theme";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

const settingsFormSchema = z.object({
  agyPath: z.string(),
  model: z.string(),
  maxSessions: z.number().int().min(1).max(1000),
  chaptersPerSession: z.number().int().min(1).max(100),
  maxReviewRounds: z.number().int().min(0).max(10),
  minLengthRatio: z.number().min(0.1).max(3),
});
type SettingsForm = z.infer<typeof settingsFormSchema>;
const NUMERIC = new Set<keyof SettingsForm>(["maxSessions", "chaptersPerSession", "maxReviewRounds", "minLengthRatio"]);

function Card({ title, description, children }: { title: string; description: string; children: ReactNode }) {
  return (
    <section className="rounded-lg border bg-card p-5">
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="mb-4 text-sm text-muted-foreground">{description}</p>
      <div className="flex flex-col gap-4">{children}</div>
    </section>
  );
}

export function SettingsPage() {
  const root = useStoryStore((s) => s.root);
  const config = useStoryStore((s) => s.config);
  const settings = useStoryStore((s) => s.snapshot?.settings);
  const agy = useStoryStore((s) => s.agy);
  const running = useStoryStore((s) => s.session.status === "running");
  const setConfig = useStoryStore((s) => s.setConfig);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const theme = useThemeActions();
  const form = useForm<SettingsForm>({ resolver: zodResolver(settingsFormSchema) });

  useEffect(() => {
    if (config && settings) {
      form.reset({
        agyPath: config.agyPath ?? "",
        model: config.model ?? "",
        maxSessions: config.maxSessions,
        chaptersPerSession: settings.chaptersPerSession,
        maxReviewRounds: settings.maxReviewRounds,
        minLengthRatio: settings.minLengthRatio,
      });
    }
  }, [config, settings, form]);

  const submit = form.handleSubmit(async (values) => {
    if (!config || !root) return;
    try {
      const next = await appConfigSet({ ...config, agyPath: values.agyPath.trim() || null, model: values.model.trim() || null, maxSessions: values.maxSessions });
      setConfig(next);
      setAgy(await agyStatus(next.agyPath ?? undefined));
      await saveSettings(root, { chaptersPerSession: values.chaptersPerSession, maxReviewRounds: values.maxReviewRounds, minLengthRatio: values.minLengthRatio });
      setSnapshot(await storySnapshot(root));
      toast.success("Đã lưu cài đặt");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được cài đặt");
    }
  });

  async function pickAgy() {
    const path = await pickAgyFile();
    if (path) form.setValue("agyPath", path, { shouldDirty: true });
  }

  const field = (name: keyof SettingsForm, label: string, hint: string, props: ComponentProps<"input"> = {}) => (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={name}>{label}</Label>
      <Input id={name} {...props} {...form.register(name, { valueAsNumber: NUMERIC.has(name) })} aria-invalid={Boolean(form.formState.errors[name])} />
      <p className={cn("text-xs", form.formState.errors[name] ? "text-destructive" : "text-muted-foreground")}>{form.formState.errors[name]?.message ?? hint}</p>
    </div>
  );

  return (
    <div className="fine-scrollbar h-full overflow-y-auto">
      <div className="mx-auto flex max-w-3xl flex-col gap-6 px-8 py-8">
        <header>
          <h1 className="text-2xl font-semibold tracking-tight">Cài đặt</h1>
          <p className="text-sm text-muted-foreground">Giao diện và App dùng chung mọi truyện; "Truyện này" ghi vào state.json của truyện đang mở.</p>
        </header>
        <Card title="Giao diện" description="Áp dụng ngay, lưu vào cấu hình app.">
          <PalettePicker value={theme.palette} onChange={(p) => void theme.setPalette(p)} />
          <div className="flex flex-col gap-1.5">
            <Label>Chế độ</Label>
            <div role="radiogroup" aria-label="Chế độ sáng tối" className="inline-flex w-fit rounded-md border p-0.5">
              {THEME_MODES.map((mode) => (
                <button key={mode} type="button" role="radio" aria-checked={theme.mode === mode} onClick={() => void theme.setMode(mode)} className={cn("rounded-[calc(var(--radius)-2px)] px-3 py-1.5 text-sm", theme.mode === mode ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:bg-accent")}>{THEME_MODE_LABELS[mode]}</button>
              ))}
            </div>
          </div>
        </Card>
        <form onSubmit={(e) => void submit(e)} className="flex flex-col gap-6">
          <Card title="App" description="Antigravity CLI và giới hạn phiên.">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="agyPath">Đường dẫn agy</Label>
              <div className="flex gap-1">
                <Input id="agyPath" {...form.register("agyPath")} placeholder="Trống = tự tìm trong PATH" />
                <Button type="button" variant="outline" size="icon" aria-label="Chọn file agy" onClick={() => void pickAgy()}><FolderSearch /></Button>
              </div>
              <p className="text-xs text-muted-foreground">{agy?.found ? `Đang dùng: ${agy.path}${agy.version ? ` (${agy.version})` : ""}` : "Chưa tìm thấy agy."}</p>
            </div>
            {field("model", "Model mặc định", "Trống = model mặc định của agy; danh sách ở dropdown trang Dịch.")}
            {field("maxSessions", "Số phiên tối đa mỗi lần Bắt đầu", "Cầu dao chống chạy vô hạn; mặc định 50.", { type: "number", min: 1, max: 1000 })}
          </Card>
          <Card title="Truyện này" description="Ghi vào state.json của truyện đang mở.">
            {field("chaptersPerSession", "Chương / phiên", "Agent dừng sau số chương này để giữ context sạch; mặc định 10.", { type: "number", min: 1, max: 100 })}
            {field("maxReviewRounds", "Số vòng soát tối đa", "Hết vòng mà chỉ còn vi phạm rule thì chốt kèm cảnh báo; mặc định 3.", { type: "number", min: 0, max: 10 })}
            {field("minLengthRatio", "Tỉ lệ ký tự dịch/raw tối thiểu", "Dưới ngưỡng coi là dịch thiếu; mặc định 0.75.", { type: "number", step: 0.05, min: 0.1, max: 3 })}
          </Card>
          <div className="flex justify-end">
            <Button type="submit" disabled={running || !form.formState.isDirty}>Lưu App + Truyện này</Button>
          </div>
        </form>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: `src/components/pages/export-page.tsx`**

```tsx
import { Download, FolderOpen } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { exportChapters, pickSaveFile, revealFolder } from "@/lib/api";
import type { ChapterRow, ExportOutcome } from "@/lib/types";
import { useStoryStore } from "@/store/story";

/** Xem trước theo thứ tự chương trong snapshot (Rust đã sort natural). */
export function previewRange(chapters: ChapterRow[], from: string, to: string) {
  const ids = chapters.map((c) => c.id);
  const start = from ? ids.indexOf(from) : 0;
  const end = to ? ids.indexOf(to) : ids.length - 1;
  if (start < 0 || end < 0 || start > end) return { valid: false, included: [] as ChapterRow[], gaps: [] as string[] };
  const slice = chapters.slice(start, end + 1);
  return { valid: true, included: slice.filter((c) => c.status === "done"), gaps: slice.filter((c) => c.status !== "done").map((c) => c.id) };
}

export function ExportPage() {
  const root = useStoryStore((s) => s.root);
  const chapters = useStoryStore((s) => s.snapshot?.chapters ?? []);
  const done = chapters.filter((c) => c.status === "done");
  const [from, setFrom] = useState(done[0]?.id ?? "");
  const [to, setTo] = useState(done[done.length - 1]?.id ?? "");
  const [result, setResult] = useState<ExportOutcome | undefined>();
  const [busy, setBusy] = useState(false);
  const preview = previewRange(chapters, from, to);

  async function run(pickPath: boolean) {
    if (!root) return;
    setBusy(true);
    try {
      const out = pickPath ? await pickSaveFile(`${from || "dau"}-${to || "cuoi"}.txt`) : undefined;
      if (pickPath && !out) return;
      const outcome = await exportChapters(root, { from: from || undefined, to: to || undefined, out });
      setResult(outcome);
      toast.success(`Đã gộp ${outcome.ids.length} chương`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Export thất bại");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fine-scrollbar h-full overflow-y-auto">
      <div className="mx-auto flex max-w-2xl flex-col gap-6 px-8 py-8">
        <header>
          <h1 className="text-2xl font-semibold tracking-tight">Export chương đã dịch</h1>
          <p className="text-sm text-muted-foreground">Gộp các chương <em>done</em> trong khoảng thành một file .txt, mỗi chương cách một dòng trống. Chương chưa xong trong khoảng được báo hổng.</p>
        </header>
        <section className="rounded-lg border bg-card p-5">
          <datalist id="chapter-ids">{chapters.map((c) => <option key={c.id} value={c.id} />)}</datalist>
          <div className="grid grid-cols-2 gap-4">
            <div className="flex flex-col gap-1.5"><Label htmlFor="from">Từ chương</Label><Input id="from" list="chapter-ids" value={from} onChange={(e) => setFrom(e.target.value)} className="font-mono" placeholder="Đầu" /></div>
            <div className="flex flex-col gap-1.5"><Label htmlFor="to">Đến chương</Label><Input id="to" list="chapter-ids" value={to} onChange={(e) => setTo(e.target.value)} className="font-mono" placeholder="Cuối" /></div>
          </div>
          <div className="mt-4 rounded-md bg-muted p-3 text-sm">
            {!preview.valid ? (
              <p className="text-destructive">Khoảng không hợp lệ: mã chương không tồn tại hoặc "từ" đứng sau "đến".</p>
            ) : (
              <>
                <p>Sẽ gộp <strong className="tabular-nums">{preview.included.length}</strong> chương done.</p>
                {preview.gaps.length > 0 && <p className="mt-1 text-status-warning">Hổng {preview.gaps.length} chương chưa done: <span className="font-mono text-xs">{preview.gaps.slice(0, 20).join(", ")}{preview.gaps.length > 20 ? "…" : ""}</span></p>}
              </>
            )}
          </div>
          <div className="mt-4 flex justify-end gap-2">
            <Button variant="outline" disabled={busy || !preview.valid || preview.included.length === 0} onClick={() => void run(true)}>Chọn nơi lưu…</Button>
            <Button disabled={busy || !preview.valid || preview.included.length === 0} onClick={() => void run(false)}><Download /> Export vào export/</Button>
          </div>
        </section>
        {result && (
          <section className="rounded-lg border border-status-done/40 bg-status-done/10 p-5 text-sm">
            <p>Đã ghi <code className="font-mono text-xs break-all">{result.outPath}</code> ({result.ids.length} chương).</p>
            {result.gaps.length > 0 && <p className="mt-1 text-status-warning">Hổng {result.gaps.length} chương: {result.gaps.join(", ")}</p>}
            <Button size="sm" variant="outline" className="mt-3" onClick={() => void revealFolder(result.outPath.replace(/[\\/][^\\/]+$/, ""))}><FolderOpen /> Mở folder</Button>
          </section>
        )}
      </div>
    </div>
  );
}
```

Thêm test `src/components/pages/export-page.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import { previewRange } from "@/components/pages/export-page";
import type { ChapterRow } from "@/lib/types";

const rows: ChapterRow[] = ["0001", "0002", "0003", "0004"].map((id, i) => ({ id, status: i === 2 ? "skipped" : "done", reviewRound: 0, reason: null, warnings: [] }));

describe("previewRange", () => {
  it("đếm done và hổng trong khoảng; rỗng = toàn bộ", () => {
    const p = previewRange(rows, "0002", "0004");
    expect(p.valid).toBe(true);
    expect(p.included.map((c) => c.id)).toEqual(["0002", "0004"]);
    expect(p.gaps).toEqual(["0003"]);
    expect(previewRange(rows, "", "").included).toHaveLength(3);
  });
  it("khoảng sai thì invalid", () => {
    expect(previewRange(rows, "0004", "0001").valid).toBe(false);
    expect(previewRange(rows, "9999", "").valid).toBe(false);
  });
});
```

(`previewRange` export ngoài component — nếu lint `react-refresh/only-export-components` báo warning, chấp nhận warning hoặc tách ra `src/lib/export-range.ts` và import lại.)

`app.tsx`: `export: ExportPage`, `settings: SettingsPage`; xoá `PAGES` placeholder, giữ `Record<Page, () => ReactElement>` với 4 trang thật.

- [ ] **Step 4: Kiểm**

Run: `npx vitest run && npm run -s typecheck && npm run -s lint`. `tauri dev`: Cài đặt → bấm Studio đổi màu ngay, đổi chế độ Tối, tắt/mở app giữ; Export xem trước số chương + hổng trước khi bấm.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui/src
git commit -m "feat(qt-ai-gui): trang Export có xem trước khoảng/hổng, trang Cài đặt với chọn bộ màu và chế độ

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

### Task 7: Dọn file cũ, cấm màu cứng, README, kiểm toàn bộ

**Files:**
- Delete: `src/components/workbench.tsx`, `progress-header.tsx`, `chapter-table.tsx`, `chapter-table.test.tsx`, `chapter-panel.tsx`, `story-form.tsx`, `settings-dialog.tsx`, `export-dialog.tsx`
- Create: `src/lib/no-hardcoded-colors.test.ts`
- Modify: `apps/qt-ai-gui/README.md`

- [ ] **Step 1: Xoá file cũ**

```bash
cd apps/qt-ai-gui && git rm -q src/components/workbench.tsx src/components/progress-header.tsx src/components/chapter-table.tsx src/components/chapter-table.test.tsx src/components/chapter-panel.tsx src/components/story-form.tsx src/components/settings-dialog.tsx src/components/export-dialog.tsx
```

- [ ] **Step 2: Test chặn màu cứng `src/lib/no-hardcoded-colors.test.ts`**

```ts
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

import { describe, expect, it } from "vitest";

const ROOT = resolve(import.meta.dirname, "../components");
const FORBIDDEN = /\b(?:bg|text|border|ring|from|to|via)-(?:zinc|slate|gray|neutral|stone|amber|red|green|blue|violet|purple|teal|emerald|orange|yellow|indigo|pink|rose|sky|cyan|lime)-\d{2,3}\b/g;

function walk(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return name === "ui" ? [] : walk(path);
    return /\.tsx$/.test(name) && !/\.test\.tsx$/.test(name) ? [path] : [];
  });
}

describe("không dùng màu cứng ngoài ui kit", () => {
  it.each(walk(ROOT))("%s", (file) => {
    const hits = readFileSync(file, "utf8").match(FORBIDDEN) ?? [];
    expect(hits).toEqual([]);
  });
});
```

- [ ] **Step 3: README app — thêm mục**

Sau mục "## Folder truyện" thêm:

```markdown
## Giao diện

Ba bộ màu (Editorial / Studio / Soft) × sáng / tối / theo hệ thống, chọn ở trang Cài đặt hoặc nút mặt trăng trên rail; lưu trong `config.json` của app. Token nằm trong `src/index.css`, mỗi tổ hợp đánh dấu `/* palette: <id> <mode> */`; `src/lib/theme-tokens.test.ts` kiểm đủ token và tương phản ≥ 4.5:1, `src/lib/no-hardcoded-colors.test.ts` chặn class màu cứng ngoài `components/ui`. Font đóng gói offline (`@fontsource-variable`).
```

- [ ] **Step 4: Kiểm toàn bộ**

```bash
cd apps/qt-ai-gui && npm run -s check && cd ../.. && cargo test -p qt-ai-gui 2>&1 | grep "test result" && npm --prefix apps/qt-ai-cli run -s golden:check
```

Expected: check xanh (0 error lint), Rust 10 pass, golden khớp. `npm run tauri build` ra bundle; mở app release chụp màn hình 4 trang × 3 bộ × 2 chế độ để duyệt tay (không có bước tự động).

- [ ] **Step 5: Commit**

```bash
git add -A apps/qt-ai-gui
git commit -m "refactor(qt-ai-gui): xoá dialog/bảng cũ, test chặn màu cứng, README giao diện

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>"
```

---

## Self-review

**Spec coverage:** Hệ theme (token 6 tổ hợp, data-palette, next-themes system, AppConfig, font offline, theme.ts thuần) → Task 1–3 ✓. Rail + 4 trang + store.page + picker tên/tiến độ + `recent_summaries` → Task 2–3 ✓. Trang Dịch (toolbar chip, list chip+tìm 280px, reader 70ch/16px prev-next, log token) → Task 4 ✓. Hồ sơ (2 cột mục lục neo, thanh dính đáy, glossary tìm, AI điền dialog) → Task 5 ✓. Export xem trước, Cài đặt 3 khối + PalettePicker → Task 6 ✓. Kiểm: theme test, token coverage, tương phản (dưới dạng vitest thay script — vẫn chạy trong `npm run check`), chapters filter test, store page test, Rust config/summaries test, chặn màu cứng → Task 1–7 ✓. Ngoài phạm vi giữ đúng.

**Placeholder scan:** không TBD/TODO; mọi component có code đầy đủ. Placeholder `PAGES` ở Task 3 được thay dần ở Task 4–6 và chốt ở Task 6.

**Type consistency:** `ChapterFilter` (chapters.ts) dùng ở store/ChapterList/TranslatePage ✓; `Page` export từ store dùng ở AppRail/App ✓; `useThemeActions` trả `palette/mode/setPalette/setMode/toggleMode` dùng ở AppRail/SettingsPage ✓; `RecentSummary` schema ↔ Rust camelCase `root/name/done/total` ✓; `appConfigSchema.palette/themeMode` ↔ Rust `palette/theme_mode` camelCase ✓; `previewRange` export từ export-page dùng trong test ✓; class `bg-status-*`, `bg-log`, `text-log-foreground`, `.reading`, `.fine-scrollbar` khai báo ở Task 1 ✓.

**Rủi ro:** giá trị oklch mới ước lượng — test tương phản sẽ bắt, chỉnh L khi đỏ. `TabsList variant="line"` cần prop tồn tại trong ui/tabs.tsx (đã có cva `line`). `datalist` trên WebView2 hoạt động; nếu không hiện gợi ý, vẫn gõ tay được.
