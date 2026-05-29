import { defineConfig } from 'vitest/config'
import { groups } from './vitest.member-namespace-alias-barrel'

export default defineConfig({
  test: {
    projects: groups.web,
  },
})
