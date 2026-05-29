import { defineConfig } from 'vitest/config'

// Unusual: projects is a single object instead of an array
// The parser handles this gracefully
export default defineConfig({
  test: {
    projects: { test: { name: 'vitest-single-object-project', include: ['vitest-single-object-project/**/*.test.ts'] } },
  },
})
