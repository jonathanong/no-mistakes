export const config = {
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-namespace-spread',
          include: ['vitest-root-namespace-spread/**/*.test.ts'],
        },
      },
    ],
  },
}

export const testConfig = {
  projects: [
    {
      test: {
        name: 'vitest-test-namespace-spread',
        include: ['vitest-test-namespace-spread/**/*.test.ts'],
      },
    },
  ],
}
