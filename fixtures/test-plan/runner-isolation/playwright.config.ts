import { defineConfig } from '@playwright/test';

declare function createPlaywrightConfig(config: unknown): unknown;

// The inner factory call is intentionally static: discovery must peel both
// wrappers without executing repository code or widening to the repository root.
export default defineConfig(createPlaywrightConfig({
  testDir: './e2e',
  projects: [
    {
      name: 'chromium',
      testMatch: '**/*.spec.mts',
    },
  ],
}));
