import { defineConfig } from '@playwright/test'
import { configs } from './playwright.root-member-import-then-export-barrel'

export default defineConfig({
  ...configs.web,
})
