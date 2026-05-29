import { defineConfig } from '@playwright/test'

export default defineConfig({
  projects: [
    {
      name: 'empty-test-match',
      testMatch: [],
    },
  ],
})
