import { defineConfig } from '@playwright/test'
import { shared } from './playwright.root-import-then-export-barrel'

export default defineConfig({
  ...shared,
})
