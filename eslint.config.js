// ESLint flat config: JS + TypeScript + Svelte rules, with formatting left to Prettier.
import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import { defineConfig } from "eslint/config";
import ts from "typescript-eslint";

export default defineConfig(
  {
    ignores: [
      "dist/",
      "node_modules/",
      "src-tauri/",
      "crates/",
      "bench/",
      "scripts/",
      "docs/",
      "target/",
      ".claude/",
    ],
  },
  js.configs.recommended,
  ts.configs.recommended,
  svelte.configs.recommended,
  prettier,
  svelte.configs.prettier,
  {
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: { ...globals.browser },
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts"],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
        extraFileExtensions: [".svelte"],
      },
    },
  },
  {
    files: ["*.config.ts", "*.config.js"],
    languageOptions: {
      globals: { ...globals.node },
    },
  },
);
