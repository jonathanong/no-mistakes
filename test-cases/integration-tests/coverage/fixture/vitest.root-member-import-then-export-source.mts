export const configs = {
  web: {
    test: {
      projects: [
        {
          test: {
            name: 'vitest-root-member-import-then-export',
            include: ['vitest-root-member-import-then-export/**/*.test.ts'],
          },
        },
      ],
    },
  },
}
