/**
 * 主题管理：支持 light / dark / system 三种模式。
 * 持久化到 localStorage，system 模式跟随 prefers-color-scheme。
 */

export type ThemeMode = "light" | "dark" | "system";

const STORAGE_KEY = "buyu-theme";

function getStoredMode(): ThemeMode {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw === "light" || raw === "dark" || raw === "system") return raw;
  } catch {
    // SSR or storage blocked
  }
  return "system";
}

function getSystemDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function applyTheme(mode: ThemeMode) {
  const isDark = mode === "dark" || (mode === "system" && getSystemDark());
  const root = document.documentElement;

  if (isDark) {
    root.classList.add("dark");
    root.style.colorScheme = "dark";
  } else {
    root.classList.remove("dark");
    root.style.colorScheme = "light";
  }
}

let currentMode = $state<ThemeMode>(getStoredMode());

// 初始化时立即应用
applyTheme(currentMode);

// 监听系统主题变化
const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
mediaQuery.addEventListener("change", () => {
  if (currentMode === "system") {
    applyTheme("system");
  }
});

export function getThemeMode(): ThemeMode {
  return currentMode;
}

export function setThemeMode(mode: ThemeMode) {
  currentMode = mode;
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    // ignore
  }
  applyTheme(mode);
}

export function resolvedDark(): boolean {
  return currentMode === "dark" || (currentMode === "system" && getSystemDark());
}
