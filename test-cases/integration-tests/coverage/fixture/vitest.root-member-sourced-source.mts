export const configs = {
  web: {
    test: {
      projects: [
        {
          test: {
            name: 'vitest-root-member-sourced',
            include: ['vitest-root-member-sourced/**/*.test.ts'],
          },
        },
      ],
    },
  },
}
