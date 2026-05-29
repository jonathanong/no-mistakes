import { defineConfig } from '@playwright/test'
import { sharedConfig } from './playwright.root-sourced-reexport-barrel'

export default defineConfig({
  ...sharedConfig,
})
