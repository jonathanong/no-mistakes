export const sharedTestConfig = {
  projects: [
    {
      test: {
        name: 'vitest-test-sourced-reexport',
        include: ['vitest-test-sourced-reexport/**/*.test.ts'],
      },
    },
  ],
  test: {
    projects: [
      {
        test: {
          name: 'vitest-nested-test-sourced-reexport',
          include: ['vitest-nested-test-sourced-reexport/**/*.test.ts'],
        },
      },
    ],
  },
}
