import "@fontsource/manrope/latin-400.css";
import "@fontsource/manrope/latin-500.css";
import "@fontsource/manrope/latin-700.css";
import "@fontsource/manrope/latin-800.css";
import "@fontsource/space-grotesk/latin-500.css";
import "@fontsource/space-grotesk/latin-700.css";
import { mount } from "svelte";
import { tauriInvoke } from "$lib/api/client";
import App from "./App.svelte";
import "./app.css";

const app = mount(App, {
  target: document.getElementById("app")!
});

requestAnimationFrame(() => {
  requestAnimationFrame(() => {
    void tauriInvoke("notify_main_window_ready").catch((error) => {
      console.error("Failed to show main window:", error);
    });
  });
});

export default app;
