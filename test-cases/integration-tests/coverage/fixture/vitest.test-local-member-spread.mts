import { defineConfig } from 'vitest/config'

const configs = {
  web: {
    projects: [{ test: { name: 'vitest-test-local-member-spread', include: ['vitest-test-local-member-spread/**/*.test.ts'] } }],
  },
}

export default defineConfig({
  test: {
    ...configs.web,
  },
})
