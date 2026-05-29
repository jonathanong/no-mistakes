import { defineConfig } from '@playwright/test'
import { makeProject } from './playwright.object-call-cycle-a-helper'

export default defineConfig({
  projects: [makeProject()],
})
