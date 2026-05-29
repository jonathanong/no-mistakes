import { defineConfig } from 'vitest/config'
import { configs } from './vitest.root-member-no-property-source'

export default defineConfig({
  ...configs.web,
  test: {
    projects: [{ test: { name: 'vitest-root-member-no-property', include: ['vitest-root-member-no-property/**/*.test.ts'] } }],
  },
})
