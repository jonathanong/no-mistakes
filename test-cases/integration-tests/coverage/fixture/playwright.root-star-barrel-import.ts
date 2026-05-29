import { defineConfig } from '@playwright/test'
import { shared } from './playwright.root-star-barrel'

export default defineConfig({
  ...shared,
})
