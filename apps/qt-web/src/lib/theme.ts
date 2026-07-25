export const themes = [
  { value: "qt", label: "QT sáng", className: "theme-qt", scheme: "light" },
  { value: "claude", label: "Claude sáng", className: "theme-claude", scheme: "light" },
  { value: "claude-dark", label: "Claude tối", className: "theme-claude-dark", scheme: "dark" },
  { value: "ktt", label: "KTT tối", className: "theme-ktt", scheme: "dark" },
  { value: "discord", label: "Discord tối", className: "theme-discord", scheme: "dark" },
] as const;

export type ThemeName = (typeof themes)[number]["value"];

export const defaultTheme: ThemeName = "qt";
const storageKey = "qt-web-theme";

const themeClasses = themes.map(({ className }) => className);

export function isThemeName(value: unknown): value is ThemeName {
  return themes.some(({ value: name }) => name === value);
}

/**
 * Áp theme lên <html>. Ngoài class màu còn gắn/gỡ `dark` để biến thể
 * `dark:` của Tailwind vẫn đúng ở ba theme nền tối.
 */
export function applyTheme(theme: ThemeName): void {
  const root = document.documentElement;
  const entry = themes.find(({ value }) => value === theme) ?? themes[0];
  root.classList.remove(...themeClasses, "dark");
  if (entry.className) root.classList.add(entry.className);
  if (entry.scheme === "dark") root.classList.add("dark");
  root.dataset.theme = entry.value;
}

export function readStoredTheme(): ThemeName {
  try {
    const saved = window.localStorage.getItem(storageKey);
    return isThemeName(saved) ? saved : defaultTheme;
  } catch {
    return defaultTheme;
  }
}

export function storeTheme(theme: ThemeName): void {
  try {
    window.localStorage.setItem(storageKey, theme);
  } catch {
    /* chế độ riêng tư chặn localStorage — không đáng để hỏng cả app */
  }
}
