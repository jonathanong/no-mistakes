import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'backend',
          include: ['backend/**/*.test.mts'],
          exclude: ['backend/**/*.mock.test.mts'],
        },
      },
      {
        test: {
          name: 'integration',
          include: ['integration/**/*.test.mts'],
        },
      },
    ],
  },
})
