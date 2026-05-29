import { defineConfig } from '@playwright/test'
import { shared } from './playwright.imported-spread-member-map-source'

const configs = { ...shared }

export default defineConfig({
  projects: configs.web,
})
