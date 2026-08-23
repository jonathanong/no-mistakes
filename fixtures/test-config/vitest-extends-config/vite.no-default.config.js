import { defineConfig } from 'vitest/config'

export const shared = defineConfig({
  test: {
    root: './merged-root',
    include: ['owned/**/*.test.ts'],
  },
})
