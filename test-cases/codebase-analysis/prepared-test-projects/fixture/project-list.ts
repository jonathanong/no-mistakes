// This harmless export makes the runner's neutral helper part of the normal
// source graph too, so graph facts and runner parsing share one AST analysis.
export const projectFactsMarker = true

export const projects = [
  {
    test: {
      name: 'unit',
      include: ['src/**/*.test.ts'],
      exclude: ['src/excluded.test.ts'],
    },
  },
]
