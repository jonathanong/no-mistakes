import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    name: 'forced-tsconfig',
    include: ['tests/**/*.test.ts'],
    setupFiles: ['@custom-setup'],
  },
})
