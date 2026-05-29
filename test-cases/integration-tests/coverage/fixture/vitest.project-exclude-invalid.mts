import { defineConfig } from 'vitest/config'

// exclude with an invalid value (unbound identifier) causes inferred_string_or_array to fail.
// This exercises the error propagation path of '?' in merge_property (objects.rs:82).
export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: 'vitest-project-exclude-invalid',
          include: ['vitest-project-exclude-invalid/**/*.test.ts'],
          // @ts-ignore
          exclude: notAStringOrArray,
        },
      },
    ],
  },
})
