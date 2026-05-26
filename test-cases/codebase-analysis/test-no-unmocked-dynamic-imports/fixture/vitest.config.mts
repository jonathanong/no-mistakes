import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['tests/**/*.test.mts', 'tests/**/*.mock.test.tsx'],
    setupFiles: ['./tests/setup-vitest.mts'],
  },
})
