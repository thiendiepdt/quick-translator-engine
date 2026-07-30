import { useCallback, useMemo, useState, type ReactNode } from "react";

import { applyTheme, readStoredTheme, storeTheme, type ThemeName } from "@/lib/theme";
import { ThemeContext, schemeOf } from "@/lib/theme-context";

export function ThemeProvider({ children }: { children: ReactNode }) {
  // Áp ngay trong initializer để không có một frame nhấp nháy sai màu.
  const [theme, setThemeState] = useState<ThemeName>(() => {
    const initial = readStoredTheme();
    applyTheme(initial);
    return initial;
  });

  const setTheme = useCallback((next: ThemeName) => {
    setThemeState(next);
    applyTheme(next);
    storeTheme(next);
  }, []);

  const value = useMemo(
    () => ({ theme, setTheme, scheme: schemeOf(theme) }),
    [setTheme, theme],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
