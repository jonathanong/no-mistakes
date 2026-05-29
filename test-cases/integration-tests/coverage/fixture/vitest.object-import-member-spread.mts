import { defineConfig } from 'vitest/config'
import { bases } from './vitest.object-import-member-spread-barrel'

export default defineConfig({
  test: {
    projects: [
      {
        ...bases.web,
      },
    ],
  },
})
