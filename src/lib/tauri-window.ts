import { getCurrentWindow } from "@tauri-apps/api/window";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export function isTauriWindowAvailable() {
  return typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
}

export function getOptionalCurrentWindow() {
  return isTauriWindowAvailable() ? getCurrentWindow() : null;
}
