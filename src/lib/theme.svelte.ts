export type Theme = "light" | "dark";
export type ThemePreference = Theme | "system";

class ThemeState {
  preference = $state<ThemePreference>("system");
  resolved = $state<Theme>("light");
  private mediaQuery: MediaQueryList | undefined;

  constructor() {
    if (typeof window !== "undefined") {
      this.mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const stored = localStorage.getItem("buyu-theme") as ThemePreference | null;
      if (stored === "light" || stored === "dark" || stored === "system") {
        this.preference = stored;
      }

      this.syncResolvedTheme();
      this.mediaQuery.addEventListener("change", () => {
        if (this.preference === "system") {
          this.syncResolvedTheme();
          this.apply();
        }
      });

      this.apply();
    }
  }

  private syncResolvedTheme() {
    const systemDark = this.mediaQuery?.matches ?? false;
    this.resolved =
      this.preference === "system" ? (systemDark ? "dark" : "light") : this.preference;
  }

  apply() {
    if (typeof document !== "undefined") {
      this.syncResolvedTheme();
      document.documentElement.setAttribute("data-theme", this.resolved);
      localStorage.setItem("buyu-theme", this.preference);
    }
  }

  toggle() {
    this.preference = this.resolved === "dark" ? "light" : "dark";
    this.apply();
  }

  set(theme: ThemePreference) {
    this.preference = theme;
    this.apply();
  }

  get isDark() {
    return this.resolved === "dark";
  }

  get isSystem() {
    return this.preference === "system";
  }
}

export const theme = new ThemeState();
