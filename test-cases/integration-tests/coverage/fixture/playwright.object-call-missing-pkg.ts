import { defineConfig } from '@playwright/test'
import { makeProject } from '@no-mistakes-test-nonexistent'

export default defineConfig({
  projects: [makeProject()],
})
