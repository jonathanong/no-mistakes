import { defineConfig } from "@playwright/test";

export default defineConfig({
  workers: process.env.CI ? 1 : undefined,
  fullyParallel: false,
  projects: [
    { name: "legacy-serial", workers: 1, fullyParallel: false },
    { name: "chromium", workers: 1 },
  ],
});
