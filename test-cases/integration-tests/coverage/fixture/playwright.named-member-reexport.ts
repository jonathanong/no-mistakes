import { defineConfig } from '@playwright/test'
import { groups } from './playwright.named-member-reexport-barrel'

export default defineConfig({
  projects: groups.web,
})
