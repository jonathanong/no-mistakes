import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests',
  projects: [
    { name: 'web', testMatch: /.*\.e2e\.ts$/v, testIgnore: [] },
    { name: 'sibling', testDir: './sibling', testMatch: '**/*.spec.ts' },
  ],
});
