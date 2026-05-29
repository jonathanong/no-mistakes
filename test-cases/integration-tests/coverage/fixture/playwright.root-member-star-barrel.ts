import { defineConfig } from '@playwright/test'
import { configs } from './playwright.root-member-star-barrel-re'

export default defineConfig({
  ...configs.web,
  projects: [{ name: 'pw-root-member-star-barrel-fallback', testMatch: ['pw-root-member-star-barrel-fallback/**/*.spec.ts'] }],
})
