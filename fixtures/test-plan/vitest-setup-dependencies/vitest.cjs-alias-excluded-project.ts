import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-alias-excluded',
    root: './cjs-commonjs-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/alias-excluded.ts',
    globalSetup: [],
  },
})
