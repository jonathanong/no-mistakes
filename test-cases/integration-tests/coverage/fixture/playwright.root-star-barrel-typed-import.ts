import { defineConfig } from '@playwright/test'
import { shared } from './playwright.root-star-barrel-typed'

export default defineConfig({
  ...shared,
})
