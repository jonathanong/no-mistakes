import { defineConfig } from '@playwright/test'

export default defineConfig({
  projects: [
    {
      name: 'inline-playwright',
      testMatch: ['inline/**/*.spec.ts'],
    },
  ],
})
