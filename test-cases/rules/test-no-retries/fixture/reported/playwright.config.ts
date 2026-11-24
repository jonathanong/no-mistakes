import { defineConfig } from "@playwright/test";

export default defineConfig({
  retries: process.env.CI ? 2 : 0,
  projects: [
    { name: "smoke", retries: 2 },
    { name: "core", retries: 1 },
  ],
});
