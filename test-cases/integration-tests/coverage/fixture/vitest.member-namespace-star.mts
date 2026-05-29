import { defineConfig } from 'vitest/config'
import { groups } from './vitest.member-namespace-star-barrel'

export default defineConfig({
  test: {
    projects: groups.web,
  },
})
