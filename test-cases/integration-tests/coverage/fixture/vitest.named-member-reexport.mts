import { defineConfig } from 'vitest/config'
import { groups } from './vitest.named-member-reexport-barrel'

export default defineConfig({
  test: {
    projects: groups.unit,
  },
})
