import { defineConfig } from '@playwright/test'

export default defineConfig({
  projects: [
    {
      name: 'chromium',
      testDir: './e2e',
      testMatch: '**/*.pw.ts',
      testIgnore: '**/*.skip.pw.ts',
    },
  ],
})
