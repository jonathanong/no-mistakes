import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'cjs-chain-excluded',
    root: './cjs-commonjs-excluded-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/chain-excluded.ts',
    globalSetup: [],
  },
})
