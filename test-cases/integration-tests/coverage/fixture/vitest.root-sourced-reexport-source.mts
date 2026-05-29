export const sharedConfig = {
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-sourced-reexport',
          include: ['vitest-root-sourced-reexport/**/*.test.ts'],
        },
      },
    ],
  },
}
