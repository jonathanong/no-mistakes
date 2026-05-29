import { defineConfig } from '@playwright/test'
import sharedConfig from './playwright.root-default-reexport-barrel'

export default defineConfig({
  ...sharedConfig,
})
