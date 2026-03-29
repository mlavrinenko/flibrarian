import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { logs } from "./lib/logs";

window.addEventListener("error", (event) => {
  logs.add({
    level: "error",
    source: "js",
    message: event.message || String(event.error),
  });
});

window.addEventListener("unhandledrejection", (event) => {
  logs.add({
    level: "error",
    source: "js",
    message:
      event.reason instanceof Error
        ? event.reason.message
        : String(event.reason),
  });
});

const target = document.getElementById("app");
if (!target) throw new Error("Missing #app element");

const app = mount(App, { target });

export default app;
