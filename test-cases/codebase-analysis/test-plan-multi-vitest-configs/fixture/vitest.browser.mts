import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'shared',
          include: ['src/browser-only.test.ts'],
        },
      },
    ],
  },
})
