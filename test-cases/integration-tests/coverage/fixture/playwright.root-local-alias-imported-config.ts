import { defineConfig } from '@playwright/test'
import { aliasImportedConfig } from './playwright.root-imported-config-helper'

export default defineConfig({
  ...aliasImportedConfig,
})
