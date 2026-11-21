import { defineConfig } from 'vitest/config'
const flag = process.env.CI === 'true'
const key = 'name'
export default defineConfig({
  test: {
    [key]: 'widgets',
    exclude: ['**/node_modules/**'],
    include: ['**/*.test.mts'],
    ...(flag ? { exclude: ['**/node_modules/**', '**/.git/**'] } : {}),
    projects: [],
  },
})
