import { defineConfig } from '@playwright/test'
import { configs } from './playwright.member-namespace-alias-fallback-barrel'

export default defineConfig({
  projects: configs.web,
})
