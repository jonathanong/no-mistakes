// Second source also exports objectStarConfig - causes ambiguity (line 27)
export const objectStarConfig = {
  test: {
    name: 'vitest-object-star-export-2',
    include: ['vitest-object-star-export-2/**/*.test.ts'],
  },
}
