import { mount } from "svelte";
import "./lib/theme.svelte";
import App from "./App.svelte";
import "./app.css";

const app = mount(App, {
  target: document.getElementById("app")!
});

/** 隐藏启动 splash：等待 workspace 就绪或超时 3 秒。 */
function hideSplash() {
  const splash = document.getElementById("boot-splash");
  if (!splash) return;

  splash.classList.add("boot-splash--hide");
  window.setTimeout(() => splash.remove(), 350);
}

let splashDismissed = false;

function dismissSplash() {
  if (splashDismissed) return;
  splashDismissed = true;
  hideSplash();
}

window.addEventListener("buyu:ready", dismissSplash, { once: true });
window.setTimeout(dismissSplash, 3000);

export default app;
