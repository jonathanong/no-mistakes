import { defineConfig } from 'vitest/config'
import { projects } from './vitest.destructured-bound-projects-helper'

export default defineConfig({
  test: {
    projects,
  },
})
