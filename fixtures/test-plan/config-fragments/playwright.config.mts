import { defineConfig } from "@playwright/test";

export default defineConfig({
  projects: [
    {
      name: "browser",
      testDir: "./e2e",
      testMatch: "**/*.spec.ts",
    },
  ],
});
