import { defineConfig } from '@playwright/test'
import { makeProject } from './playwright.object-call-import-helper'

export default defineConfig({
  projects: [makeProject()],
})
