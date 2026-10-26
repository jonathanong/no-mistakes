import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['src/**/*.test.mts'],
    setupFiles: ['./src/setup.mts'],
  },
})
