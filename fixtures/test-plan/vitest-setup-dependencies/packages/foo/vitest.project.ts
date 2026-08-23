import { defineProject } from 'vitest/config'

// This standalone project intentionally has no `root`: Vitest defaults it to
// this config file's directory, not the aggregate config's directory.
export default defineProject({
  test: {
    name: 'nested-string-default-root',
    include: ['tests/**/*.test.ts'],
    setupFiles: './setup.ts',
    globalSetup: './global.ts',
  },
})
