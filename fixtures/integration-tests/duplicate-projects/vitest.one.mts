import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    name: 'unit',
    include: ['one/**/*.test.ts'],
  },
})
