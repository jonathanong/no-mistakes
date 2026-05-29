import { defineConfig } from 'vitest/config'

const spreadTestOptions = {
  projects: [
    {
      test: {
        name: 'spread-test-options',
        include: ['spread-test-options/**/*.test.ts'],
      },
    },
  ],
}

export default defineConfig({
  test: {
    ...spreadTestOptions,
  },
})
