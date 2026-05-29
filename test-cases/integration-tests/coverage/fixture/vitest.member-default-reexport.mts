import { defineConfig } from 'vitest/config'
import groups from './vitest.member-default-reexport-barrel'

export default defineConfig({
  test: {
    projects: groups.web,
  },
})
