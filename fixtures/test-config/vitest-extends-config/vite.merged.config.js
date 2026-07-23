import { defineConfig, mergeConfig } from 'vitest/config'

const shared = defineConfig({
  test: {
    root: './merged-root',
    include: ['owned/**/*.test.ts'],
    setupFiles: './merged-setup.ts',
  },
})

export default mergeConfig(
  shared,
  defineConfig({
    test: {
      exclude: ['ignored/**'],
    },
  }),
)
