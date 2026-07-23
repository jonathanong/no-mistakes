import { defineProject } from 'vitest/config'

export default defineProject({
  test: {
    name: 'string-project',
    root: './string-project',
    include: ['**/*.test.ts'],
    setupFiles: './setup/string.ts',
    globalSetup: './setup/global.cjs',
  },
})
