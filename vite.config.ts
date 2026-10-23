/// <reference types="vitest" />
import { defineConfig, type Plugin } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { gzipSync } from "node:zlib";

// Prints the gzipped size of everything index.html loads up front (scripts, stylesheets, modulepreloads).
// Lazy chunks reached only through dynamic import() are not counted.
function firstPaintSize(): Plugin {
  let outDir = "dist";
  return {
    name: "swiftpdf-first-paint-size",
    apply: "build",
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
    },
    closeBundle() {
      const html = readFileSync(resolve(outDir, "index.html"), "utf8");
      const urls = [...html.matchAll(/<(?:script|link)[^>]+(?:src|href)="\/?([^"]+\.(?:js|css))"/g)].map(
        (match) => match[1],
      );
      let jsBytes = 0;
      let cssBytes = 0;
      for (const url of urls) {
        const gzipped = gzipSync(readFileSync(resolve(outDir, url)), { level: 9 }).length;
        if (url.endsWith(".css")) {
          cssBytes += gzipped;
        } else {
          jsBytes += gzipped;
        }
      }
      const kb = (bytes: number) => (bytes / 1024).toFixed(2) + " kB";
      console.log(
        `\nFirst-paint bundle (gzip): JS ${kb(jsBytes)} + CSS ${kb(cssBytes)} = ${kb(jsBytes + cssBytes)} ` +
          `(${urls.length} files)`,
      );
    },
  };
}

// Tauri expects a fixed dev port and must not have its own files watched by Vite.
export default defineConfig({
  plugins: [svelte(), firstPaintSize()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "localhost",
    watch: {
      ignored: ["**/src-tauri/**", "**/crates/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    // WebView2 on Windows 10/11 is Chromium-based and evergreen.
    target: "chrome105",
    minify: "esbuild",
    sourcemap: false,
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
  },
});
