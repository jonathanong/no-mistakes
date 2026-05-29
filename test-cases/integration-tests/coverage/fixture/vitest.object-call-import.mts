import { defineConfig } from 'vitest/config'
import { makeProject } from './vitest.object-call-import-helper'

export default defineConfig({
  test: {
    projects: [makeProject()],
  },
})
