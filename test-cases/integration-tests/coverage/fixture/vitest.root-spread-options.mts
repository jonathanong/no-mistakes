import { defineConfig } from 'vitest/config'

const shared = {
  test: {
    include: ['vitest-root-spread-options/**/*.test.ts'],
    exclude: ['vitest-root-spread-options/**/*.skip.ts'],
    projects: [
      {
        test: {
          name: 'vitest-root-spread-options',
        },
      },
    ],
  },
}

export default defineConfig({
  test: {
    include: ['vitest-root-spread-options-stale/**/*.test.ts'],
  },
  ...shared,
})
