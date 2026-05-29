import { defineConfig } from '@playwright/test'
import shared from './playwright.root-imported-config-helper'

export default defineConfig({
  ...shared,
})
