import { defineConfig } from '@playwright/test'
import { groups } from './playwright.member-import-then-export-barrel'

export default defineConfig({
  projects: groups.web,
})
