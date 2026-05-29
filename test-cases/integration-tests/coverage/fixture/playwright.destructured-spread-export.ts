import { defineConfig } from '@playwright/test'
import { web } from './playwright.destructured-spread-export-helper'

export default defineConfig({
  projects: web,
})
