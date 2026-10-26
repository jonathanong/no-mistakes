import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests',
  projects: [
    { name: 'web', testMatch: ['**/*.extra.ts', /.*\.e2e\.tsx?/i], testIgnore: ['**/glob-skip/**', /regex-skip/i] },
    { name: 'sibling', testDir: './sibling', testMatch: '**/*.spec.ts' },
  ],
});
