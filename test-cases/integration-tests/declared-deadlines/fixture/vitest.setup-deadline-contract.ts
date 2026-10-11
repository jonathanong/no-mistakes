import { defineConfig } from "vitest/config";
// Nested extends retains setup planning without becoming SDK deadline inheritance.
export default defineConfig({
  test: {
    testTimeout: 30000,
    hookTimeout: 1,
    setupFiles: "./root-setup.ts",
    projects: [
      { test: { name: "nested-true", extends: true } },
      { test: { name: "nested-false", extends: false } },
      { test: { name: "nested-file", extends: "./base.config.ts" } },
      { extends: false, test: { name: "outer-false" } },
      { extends: "./base.config.ts", test: { name: "outer-file" } },
    ],
  },
});
