export const nestedRootConfig = {
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-sourced-reexport-nested',
          include: ['vitest-root-sourced-reexport-nested/**/*.test.ts'],
        },
      },
    ],
  },
}
