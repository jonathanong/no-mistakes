import { defineConfig } from 'vitest/config'
import { configs } from '@missing-no-mistakes-pkg'

export default defineConfig({
  ...configs.web,
  test: {
    projects: [{ test: { name: 'vitest-root-member-missing-import', include: ['vitest-root-member-missing-import/**/*.test.ts'] } }],
  },
})
