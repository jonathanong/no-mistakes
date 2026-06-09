import { defineConfig } from '@playwright/test';
import { createPlaywrightConfig } from './config-helper';

// The config object literal is passed through a wrapper helper call. Static
// parsing recovers the literal's `testDir`/`projects`, but options the helper
// adds internally (e.g. testIdAttribute) remain invisible.
export default defineConfig(
  createPlaywrightConfig({
    testDir: './wrapped-tests',
    projects: [{ name: 'chromium', testMatch: '**/*.wrapped.ts' }],
  }),
);
