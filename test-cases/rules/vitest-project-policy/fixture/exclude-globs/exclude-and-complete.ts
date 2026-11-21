import { defineConfig } from 'vitest/config'
const flag = process.env.CI === 'true'
export default defineConfig({
  test: {
    include: ['**/*.test.mts'],
    ...(flag && { exclude: ['**/node_modules/**', '**/.git/**'] }),
  },
})
