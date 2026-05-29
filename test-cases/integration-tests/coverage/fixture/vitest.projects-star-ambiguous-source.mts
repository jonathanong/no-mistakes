// Second source with same export name as the main source - causes ambiguity when imported via star
export const myStarProjects = [
  {
    test: {
      name: 'vitest-projects-star-ambiguous',
      include: ['vitest-projects-star-ambiguous/**/*.test.ts'],
    },
  },
]
