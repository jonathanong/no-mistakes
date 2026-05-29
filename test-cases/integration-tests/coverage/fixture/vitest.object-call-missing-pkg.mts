import { defineConfig } from 'vitest/config'
import { makeProject } from '@no-mistakes-test-nonexistent'

export default defineConfig({
  test: {
    projects: [makeProject()],
  },
})
