import { defineConfig } from 'vitest/config'

const shared = defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-define-config-spread',
          include: ['vitest-root-define-config-spread/**/*.test.ts'],
        },
      },
    ],
  },
})

export default defineConfig({
  ...shared,
})
