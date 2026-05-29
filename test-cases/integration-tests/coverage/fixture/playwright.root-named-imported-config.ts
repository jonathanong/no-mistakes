import { defineConfig } from '@playwright/test'
import { namedImportedConfig } from './playwright.root-imported-config-helper'

export default defineConfig({
  ...namedImportedConfig,
})
