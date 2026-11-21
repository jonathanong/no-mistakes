import { defineConfig } from 'vitest/config'
const flag = process.env.CI === 'true'
const key = 'exclude'
export default defineConfig({
  test: {
    [key]: [],
    include: ['**/*.test.mts'],
    ...(flag
      ? { exclude: ['**/node_modules/**', '**/.git/**'] }
      : { exclude: ['**/node_modules/**', '**/.git/**'] }),
    projects: [],
  },
})
