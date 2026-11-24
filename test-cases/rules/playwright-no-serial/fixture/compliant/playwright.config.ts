import { defineConfig } from "@playwright/test";

export default defineConfig({
  workers: 4,
  fullyParallel: true,
  projects: [{ name: "chromium", workers: 2, fullyParallel: true }],
});
