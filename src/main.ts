import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import App from "./App.svelte";

const target = document.getElementById("app");
if (target) {
  mount(App, { target });
}

// The window is already visible (dark background set natively). After the first frame with content is painted,
// tell the backend so it can record the content-ready startup mark.
requestAnimationFrame(() => {
  requestAnimationFrame(() => {
    invoke("window_ready").catch((error) => console.error("window_ready failed", error));
  });
});
