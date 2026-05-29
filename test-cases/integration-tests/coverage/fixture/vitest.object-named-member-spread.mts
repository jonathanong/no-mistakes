import { defineConfig } from 'vitest/config'
import { bases } from './vitest.object-named-member-spread-source'

export default defineConfig({
  test: {
    projects: [
      {
        ...bases.web,
      },
    ],
  },
})
