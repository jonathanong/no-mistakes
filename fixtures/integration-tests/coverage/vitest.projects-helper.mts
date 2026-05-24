export const webProjects = () => [
  {
    test: {
      name: 'web',
      include: ['web/**/*.test.ts'],
      exclude: ['web/**/*.skip.ts'],
    },
  },
]

export const apiProjects = [
  {
    test: {
      name: 'api',
      include: ['api/**/*.test.ts'],
    },
  },
]
