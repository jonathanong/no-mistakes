import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-named-reexport-excluded',
    root: './cjs-commonjs-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/named-reexport-excluded.ts',
    globalSetup: [],
  },
})
