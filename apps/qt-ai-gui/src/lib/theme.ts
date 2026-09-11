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
  /** Màu xem trước cho thẻ chọn (không cần khớp token tuyệt đối): [nền, chữ, nhấn]. */
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

export const THEME_MODE_LABELS: Record<ThemeMode, string> = {
  light: "Sáng",
  dark: "Tối",
  system: "Theo hệ thống",
};

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
