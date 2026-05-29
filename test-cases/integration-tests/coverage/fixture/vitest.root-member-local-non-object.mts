import { defineConfig } from 'vitest/config'

const configs = {
  web: 'not-an-object',
}

export default defineConfig({
  ...configs.web,
  test: {
    projects: [{ test: { name: 'vitest-root-member-local-non-object', include: ['vitest-root-member-local-non-object/**/*.test.ts'] } }],
  },
})
