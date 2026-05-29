import { defineConfig } from 'vitest/config'
import { groups } from './vitest.named-member-projects-helper'

export default defineConfig({
  test: {
    projects: groups.unit,
  },
})
