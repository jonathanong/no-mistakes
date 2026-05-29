import { defineConfig } from 'vitest/config'
import { groups } from './vitest.member-import-then-export-barrel'

export default defineConfig({
  test: {
    projects: groups.web,
  },
})
