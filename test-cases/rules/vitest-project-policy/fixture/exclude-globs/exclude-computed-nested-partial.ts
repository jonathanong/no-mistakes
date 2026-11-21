import { defineConfig } from 'vitest/config'
const outer = process.env.CI === 'true'
const inner = process.env.NODE_ENV === 'test'
const key = 'exclude'
export default defineConfig({
  test: {
    [key]: [],
    include: ['**/*.test.mts'],
    ...(outer
      ? { ...(inner ? { exclude: ['**/node_modules/**'] } : {}) }
      : { exclude: ['**/node_modules/**', '**/.git/**'] }),
    projects: [],
  },
})
