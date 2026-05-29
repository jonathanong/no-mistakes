export function makeShared() {
  return {
    test: {
      projects: [
        {
          test: {
            name: 'vitest-root-call-import',
            include: ['vitest-root-call-import/**/*.test.ts'],
          },
        },
      ],
    },
  }
}
