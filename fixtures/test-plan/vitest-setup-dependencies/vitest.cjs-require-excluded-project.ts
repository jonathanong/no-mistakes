import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-require-excluded',
    root: './cjs-require-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/require-excluded.ts',
    globalSetup: [],
  },
})
