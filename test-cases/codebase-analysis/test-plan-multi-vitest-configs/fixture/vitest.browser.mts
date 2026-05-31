import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'shared',
          // Intentionally different from src/shared.test.ts. The no-mistakes
          // policy fixture replaces this include while preserving the config.
          include: ['src/browser-only.test.ts'],
        },
      },
    ],
  },
})
