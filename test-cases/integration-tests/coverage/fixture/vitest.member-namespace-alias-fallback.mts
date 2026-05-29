import { defineConfig } from 'vitest/config'
import { groups } from './vitest.member-namespace-alias-fallback-barrel'

export default defineConfig({
  test: {
    projects: groups.web,
  },
})
