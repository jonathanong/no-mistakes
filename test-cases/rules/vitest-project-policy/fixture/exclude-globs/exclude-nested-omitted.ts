import { defineConfig } from 'vitest/config'
const outer = process.env.CI === 'true'
const inner = process.env.NODE_ENV === 'test'
export default defineConfig({
  test: {
    include: ['**/*.test.mts'],
    exclude: [],
    ...(outer
      ? { ...(inner ? { exclude: ['**/node_modules/**', '**/.git/**'] } : {}) }
      : { exclude: ['**/node_modules/**', '**/.git/**'] }),
    projects: [],
  },
})
