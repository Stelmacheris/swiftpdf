import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import App from "./App.svelte";
import "./app.css";

const target = document.getElementById("app");
if (target) {
  mount(App, { target });
}

// The window starts hidden; show it once the first frame has been painted.
requestAnimationFrame(() => {
  requestAnimationFrame(() => {
    invoke("window_ready").catch((error) => console.error("window_ready failed", error));
  });
});
