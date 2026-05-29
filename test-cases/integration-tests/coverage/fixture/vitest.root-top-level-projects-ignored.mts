import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-test-projects',
          include: ['vitest-root-test-projects/**/*.test.ts'],
        },
      },
    ],
  },
  projects: [
    {
      test: {
        name: 'vitest-root-top-level-projects',
        include: ['vitest-root-top-level-projects/**/*.test.ts'],
      },
    },
  ],
})
