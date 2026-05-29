import { defineConfig } from 'vitest/config'

const base = {
  test: {
    include: ['vitest-project-test-override-clears-stale/**/*.test.ts'],
    exclude: ['vitest-project-test-override-clears-stale/**/*.skip.ts'],
  },
}

export default defineConfig({
  test: {
    projects: [
      {
        ...base,
        test: {
          name: 'vitest-project-test-override-clears',
        },
      },
    ],
  },
})
