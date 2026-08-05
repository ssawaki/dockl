import js from "@eslint/js";
import globals from "globals";
import svelte from "eslint-plugin-svelte";
import tseslint from "typescript-eslint";
import prettier from "eslint-config-prettier";
import svelteConfig from "./svelte.config.js";

export default tseslint.config(
  {
    ignores: ["build/", "package/", ".svelte-kit/", "node_modules/", "src-tauri/", "static/"],
  },

  js.configs.recommended,
  tseslint.configs.recommended,
  svelte.configs.recommended,

  // Turns off every rule that would fight Prettier over formatting. Must stay last among
  // the shared configs so it can override what they enable.
  prettier,
  svelte.configs.prettier,

  {
    languageOptions: {
      // The app runs in a Tauri webview, and the build/config scripts run in Node.
      globals: { ...globals.browser, ...globals.node },
    },
  },

  {
    // svelte-eslint-parser handles the markup, but the <script lang="ts"> blocks still
    // need the TypeScript parser, and it needs svelte.config.js to resolve preprocessors.
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: [".svelte"],
        svelteConfig,
      },
    },
  },
);
