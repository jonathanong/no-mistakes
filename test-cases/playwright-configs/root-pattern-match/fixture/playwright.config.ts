import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './tests',
  testMatch: ['**/*.extra.ts', /.*\.e2e\.tsx?/i],
  testIgnore: ['**/glob-skip/**', /regex-skip/i],
  projects: [
    { name: 'web' },
    { name: 'sibling', testDir: './sibling', testMatch: '**/*.spec.ts' },
  ],
});
