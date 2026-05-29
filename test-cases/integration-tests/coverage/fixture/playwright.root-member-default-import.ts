import { defineConfig } from '@playwright/test'
import defaultConfigs from './playwright.root-member-default-import-source'

export default defineConfig({
  ...defaultConfigs.web,
})
