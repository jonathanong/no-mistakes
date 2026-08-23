import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-named-object-excluded',
    root: './cjs-commonjs-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/named-object-excluded.ts',
    globalSetup: [],
  },
})
