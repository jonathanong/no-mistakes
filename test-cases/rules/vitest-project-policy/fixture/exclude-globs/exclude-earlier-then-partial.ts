import { defineConfig } from 'vitest/config'
const flag = process.env.CI === 'true'
export default defineConfig({
  test: {
    exclude: ['**/node_modules/**', '**/.git/**'],
    include: ['**/*.test.mts'],
    ...(flag ? { exclude: ['**/node_modules/**'] } : {}),
  },
})
