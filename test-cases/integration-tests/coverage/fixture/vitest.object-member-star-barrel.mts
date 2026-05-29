import { defineConfig } from 'vitest/config'
import { bases } from './vitest.object-member-star-barrel-re'

export default defineConfig({
  test: {
    projects: [
      {
        ...bases.web,
        test: { name: 'vitest-object-member-star-barrel-fallback' },
      },
    ],
  },
})
