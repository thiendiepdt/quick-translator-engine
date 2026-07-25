import { createContext, useContext } from "react";

import { defaultTheme, themes, type ThemeName } from "@/lib/theme";

export interface ThemeContextValue {
  theme: ThemeName;
  setTheme: (theme: ThemeName) => void;
  /** "light" | "dark" — dùng cho các thư viện cần biết nền sáng hay tối. */
  scheme: "light" | "dark";
}

export const ThemeContext = createContext<ThemeContextValue>({
  theme: defaultTheme,
  setTheme: () => {},
  scheme: "light",
});

export function useTheme(): ThemeContextValue {
  return useContext(ThemeContext);
}

export function schemeOf(theme: ThemeName): "light" | "dark" {
  return themes.find(({ value }) => value === theme)?.scheme ?? "light";
}
