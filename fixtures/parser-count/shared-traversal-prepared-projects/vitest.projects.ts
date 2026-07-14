// Intentionally imported by the runner config: discovery and graph facts must
// share this helper parse instead of analyzing it independently.
export const projects = [
  {
    test: {
      name: 'unit',
      include: ['src/**/*.test.ts'],
      exclude: ['src/excluded.test.ts'],
    },
  },
]
