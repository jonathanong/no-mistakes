import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    name: 'resolved',
    include: ['resolved/**/*.test.ts'],
    setupFiles: './setup/resolved.ts',
  },
})
