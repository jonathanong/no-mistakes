import { defineConfig } from '@playwright/test'
import { configs } from './playwright.root-member-sourced-barrel'

export default defineConfig({
  ...configs.web,
})
