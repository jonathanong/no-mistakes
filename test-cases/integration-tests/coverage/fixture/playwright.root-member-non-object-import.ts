import { defineConfig } from '@playwright/test'
import { configs } from './playwright.root-member-non-object-import-source'

export default defineConfig({
  ...configs.web,
  projects: [{ name: 'pw-root-member-non-object-import', testMatch: ['pw-root-member-non-object-import/**/*.spec.ts'] }],
})
