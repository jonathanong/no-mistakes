import { defineConfig } from 'vitest/config'
import { configs } from './vitest.root-member-star-barrel-re'

export default defineConfig({
  ...configs.web,
  test: {
    projects: [{ test: { name: 'vitest-root-member-star-barrel-fallback', include: ['vitest-root-member-star-barrel-fallback/**/*.test.ts'] } }],
  },
})
