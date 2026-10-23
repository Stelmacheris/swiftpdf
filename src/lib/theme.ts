// Theme selection (theme.md section 1): the setting is stored as-is; the page only ever sees "dark" or "light".

export type ThemeSetting = "dark" | "light" | "system";
export type ResolvedTheme = "dark" | "light";

export const DEFAULT_THEME_SETTING: ThemeSetting = "dark";

const SYSTEM_DARK_QUERY = "(prefers-color-scheme: dark)";

export function resolveTheme(setting: ThemeSetting, systemPrefersDark: boolean): ResolvedTheme {
  if (setting === "system") {
    return systemPrefersDark ? "dark" : "light";
  }
  return setting;
}

export function applyTheme(
  setting: ThemeSetting,
  root: HTMLElement = document.documentElement,
): ResolvedTheme {
  const systemPrefersDark = window.matchMedia(SYSTEM_DARK_QUERY).matches;
  const theme = resolveTheme(setting, systemPrefersDark);
  root.dataset.theme = theme;
  return theme;
}

// Re-applies the theme when Windows switches between dark and light. Returns a function that stops listening.
export function followSystemTheme(onChange: (theme: ResolvedTheme) => void): () => void {
  const query = window.matchMedia(SYSTEM_DARK_QUERY);
  const listener = (event: MediaQueryListEvent) => onChange(event.matches ? "dark" : "light");
  query.addEventListener("change", listener);
  return () => query.removeEventListener("change", listener);
}
