// ESLint 8 (legacy config): ESLint 9 and typescript-eslint 7+ need Node >= 18.18, the toolchain is pinned to 18.12.
module.exports = {
  root: true,
  env: { browser: true, es2022: true },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: "module",
    extraFileExtensions: [".svelte"],
    warnOnUnsupportedTypeScriptVersion: false,
  },
  plugins: ["@typescript-eslint"],
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:svelte/recommended",
    "prettier",
  ],
  overrides: [
    {
      files: ["*.svelte"],
      parser: "svelte-eslint-parser",
      parserOptions: { parser: "@typescript-eslint/parser" },
    },
    {
      files: ["*.config.ts", "*.config.js", ".eslintrc.cjs"],
      env: { node: true, browser: false },
    },
  ],
  ignorePatterns: ["dist/", "node_modules/", "src-tauri/", "crates/", "bench/", "scripts/", "target/"],
};
