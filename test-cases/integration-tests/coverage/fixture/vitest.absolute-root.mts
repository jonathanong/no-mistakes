import { defineConfig } from 'vitest/config'

// Uses an absolute path for project root
// Covers the absolute path branch in vitest.rs to_project()
export default defineConfig({
  test: {
    projects: [
      {
        name: 'vitest-absolute-root',
        root: '/tmp/absolute-root',
        include: ['**/*.test.ts'],
      },
    ],
  },
})
