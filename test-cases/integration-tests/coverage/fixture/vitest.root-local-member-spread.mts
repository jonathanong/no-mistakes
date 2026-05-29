import { defineConfig } from 'vitest/config'

const configs = {
  web: {
    test: {
      projects: [{ test: { name: 'vitest-root-local-member-spread', include: ['vitest-root-local-member-spread/**/*.test.ts'] } }],
    },
  },
}

export default defineConfig({
  ...configs.web,
})
