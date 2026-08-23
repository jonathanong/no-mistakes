import * as vite from 'vitest/config'

const shared = vite.defineConfig({
  test: {
    root: './merged-root',
    include: ['owned/**/*.test.ts'],
    setupFiles: './merged-setup.ts',
  },
})

export default vite.mergeConfig(
  shared,
  vite.defineConfig({
    cacheDir: '.merged-member-cache',
  }),
)
