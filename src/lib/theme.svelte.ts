export type Theme = "light" | "dark";

class ThemeState {
  current = $state<Theme>("light");

  constructor() {
    // Check system preference on init
    if (typeof window !== "undefined") {
      const stored = localStorage.getItem("buyu-theme") as Theme | null;
      if (stored) {
        this.current = stored;
      } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        this.current = "dark";
      }
      this.apply();
    }
  }

  apply() {
    if (typeof document !== "undefined") {
      document.documentElement.setAttribute("data-theme", this.current);
      localStorage.setItem("buyu-theme", this.current);
    }
  }

  toggle() {
    this.current = this.current === "light" ? "dark" : "light";
    this.apply();
  }

  set(theme: Theme) {
    this.current = theme;
    this.apply();
  }

  get isDark() {
    return this.current === "dark";
  }
}

export const theme = new ThemeState();
