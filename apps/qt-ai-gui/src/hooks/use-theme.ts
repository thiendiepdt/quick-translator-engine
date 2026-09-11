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

  async function save(patch: { palette?: string; themeMode?: string }) {
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
