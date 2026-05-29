import { defineConfig } from 'vitest/config'
import { configs } from './vitest.root-member-non-object-import-source'

export default defineConfig({
  ...configs.web,
  test: {
    projects: [{ test: { name: 'vitest-root-member-non-object-import', include: ['vitest-root-member-non-object-import/**/*.test.ts'] } }],
  },
})
