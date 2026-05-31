import { defineConfig } from '@playwright/test'

export default defineConfig({
  name: 'top-level-policy',
  testDir: './e2e',
  testMatch: '**/*.spec.ts',
})
