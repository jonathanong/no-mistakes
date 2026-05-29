export const starBarrelConfig = {
  test: {
    projects: [
      {
        test: {
          name: 'vitest-root-spread-star-barrel',
          include: ['vitest-root-spread-star-barrel/**/*.test.ts'],
        },
      },
    ],
  },
}
