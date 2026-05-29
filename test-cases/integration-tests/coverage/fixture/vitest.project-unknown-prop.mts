import { defineConfig } from 'vitest/config'

// Project with an unrecognized property - covers _ => {} arm in merge_property (objects.rs)
export default defineConfig({
  test: {
    projects: [
      {
        name: 'vitest-unknown-prop',
        include: ['vitest-unknown-prop/**/*.test.ts'],
        someUnknownProperty: 'ignored',
      } as any,
    ],
  },
})
