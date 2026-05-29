import { defineConfig } from 'vitest/config'
import { makeProject } from './vitest.object-call-cycle-a-helper'

export default defineConfig({
  test: {
    projects: [makeProject()],
  },
})
