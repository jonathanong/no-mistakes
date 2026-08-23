import { defineConfig, mergeConfig } from 'vitest/config'

const shared = defineConfig({
  test: {
    root: './merged-root',
    include: ['owned/**/*.test.ts'],
  },
})
const extra = defineConfig({
  cacheDir: '.merged-spread-cache',
})
const configs = [shared, extra]

export default mergeConfig(...configs)
