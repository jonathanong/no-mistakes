import { defineConfig } from '@playwright/test';

export default defineConfig({
  webServer: [
    { command: 'node scripts/api-start.mts' },
    { command: 'node scripts/worker-start.mts' },
  ],
});
