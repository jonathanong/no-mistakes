import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-cycle-excluded',
    root: './cjs-commonjs-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/cycle-excluded.ts',
    globalSetup: [],
  },
})
