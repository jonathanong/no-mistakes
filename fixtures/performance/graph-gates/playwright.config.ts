import { defineConfig } from "@playwright/test";

export default defineConfig({
  webServer: [
    { command: "node scripts/api-start.ts" },
    { command: "node scripts/worker-start.ts" },
  ],
});
