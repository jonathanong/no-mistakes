import { defineConfig } from '@playwright/test'
import { makeProject } from './playwright.object-call-import-non-fn-helper'

export default defineConfig({
  projects: [makeProject()],
})
