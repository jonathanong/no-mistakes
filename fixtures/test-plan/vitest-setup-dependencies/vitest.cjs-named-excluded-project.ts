import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-named-excluded',
    root: './cjs-named-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/named-excluded.ts',
    globalSetup: [],
  },
})
