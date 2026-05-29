import { defineConfig } from 'vitest/config'
import { inner, missingProject } from './vitest.export-destructure-edge-helper'

// Uses imports that come from edge-case destructuring patterns in the helper
export default defineConfig({
  test: {
    projects: [inner, missingProject].filter(Boolean),
  },
})
