export const sharedConfig = {
  web: {
    test: {
      projects: [
        {
          test: {
            name: 'vitest-root-spread-named-member',
            include: ['vitest-root-spread-named-member/**/*.test.ts'],
          },
        },
      ],
    },
  },
}
