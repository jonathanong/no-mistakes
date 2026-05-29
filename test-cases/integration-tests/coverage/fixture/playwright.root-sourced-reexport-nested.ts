import { defineConfig } from '@playwright/test'
import { nestedRootConfig } from './playwright.root-reexport-nested'

export default defineConfig({
  ...nestedRootConfig,
})
