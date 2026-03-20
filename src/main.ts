import "@fontsource/manrope/latin-400.css";
import "@fontsource/manrope/latin-500.css";
import "@fontsource/manrope/latin-700.css";
import "@fontsource/manrope/latin-800.css";
import "@fontsource/space-grotesk/latin-500.css";
import "@fontsource/space-grotesk/latin-700.css";
import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";

const app = mount(App, {
  target: document.getElementById("app")!
});

export default app;
