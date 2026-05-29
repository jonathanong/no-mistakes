import { defineConfig } from 'vitest/config'

const bases = {
  web: [
    {
      test: {
        name: 'vitest-member-local-direct',
        include: ['vitest-member-local-direct/**/*.test.ts'],
      },
    },
  ],
}

export default defineConfig({
  test: {
    projects: bases.web,
  },
})
