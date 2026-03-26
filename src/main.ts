import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";

const app = mount(App, {
  target: document.getElementById("app")!
});

requestAnimationFrame(() => {
  const splash = document.getElementById("boot-splash");
  if (!splash) {
    return;
  }

  splash.classList.add("boot-splash--hide");
  window.setTimeout(() => {
    splash.remove();
  }, 220);
});

export default app;
