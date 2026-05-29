// NOT a function - this should cause the "function not found" branch
export const makeConfig = {
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-call-import-non-fn',
          include: ['vitest-root-call-import-non-fn/**/*.test.ts'],
        },
      },
    ],
  },
}
