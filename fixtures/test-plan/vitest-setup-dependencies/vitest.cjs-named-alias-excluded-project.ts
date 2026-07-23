import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-named-alias-excluded',
    root: './cjs-commonjs-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/named-alias-excluded.ts',
    globalSetup: [],
  },
})
