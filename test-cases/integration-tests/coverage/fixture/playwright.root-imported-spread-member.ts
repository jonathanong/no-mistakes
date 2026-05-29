import { defineConfig } from '@playwright/test'
import { shared } from './playwright.root-imported-spread-member-source'

const configs = { ...shared }

export default defineConfig({
  ...configs.web,
})
