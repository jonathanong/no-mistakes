import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  testMatch: getMatcher(),
  use: { baseURL: 'http://localhost:3000' },
});

// Runtime configuration is valid, but static ownership is deliberately uncertain.
function getMatcher() {
  return new RegExp(process.env.TEST_MATCH || '.*\\.spec\\.ts$');
}
