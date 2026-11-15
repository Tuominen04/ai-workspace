// The colour theme: follow the system, or a fixed light or dark.
//
// The choice is a display preference of this computer's, not workspace state, so it is kept
// in the webview's local storage rather than in the workspace folder. Storage can be missing
// or refuse writes; the app then simply follows the system.

export const THEMES = ["system", "light", "dark"] as const;
export type Theme = (typeof THEMES)[number];

const KEY = "ai-workspace.theme";

export function isTheme(value: unknown): value is Theme {
  return typeof value === "string" && (THEMES as readonly string[]).includes(value);
}

export function savedTheme(): Theme {
  try {
    const value = window.localStorage.getItem(KEY);
    return isTheme(value) ? value : "system";
  } catch {
    return "system";
  }
}

/** Applies `theme` to the page and remembers it. "system" removes any fixed choice. */
export function applyTheme(theme: Theme): void {
  const root = document.documentElement;
  if (theme === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", theme);
  try {
    if (theme === "system") window.localStorage.removeItem(KEY);
    else window.localStorage.setItem(KEY, theme);
  } catch {
    // Not remembered; it still applies for this run.
  }
}

export function themeLabel(theme: Theme): string {
  return { system: "System", light: "Light", dark: "Dark" }[theme];
}
