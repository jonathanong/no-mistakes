import { defineConfig } from 'vitest/config'
import { groups } from './vitest.member-nested'

export default defineConfig({
  test: {
    projects: groups.web,
  },
})
