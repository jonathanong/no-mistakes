export const sharedConfig = {
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-default-reexport',
          include: ['vitest-root-default-reexport/**/*.test.ts'],
        },
      },
    ],
  },
}
