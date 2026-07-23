export default {
  test: {
    projects: [
      // Both owners are intentionally under built-in/configured skipped dirs:
      // neither may produce a lazy Vitest setup project or graph edge.
      {
        test: {
          name: 'builtin-skipped',
          include: ['fixtures/**/*.test.ts'],
          setupFiles: './fixtures/setup.ts',
        },
      },
      {
        test: {
          name: 'configured-skipped',
          include: ['custom-skipped/**/*.test.ts'],
          setupFiles: './custom-skipped/setup.ts',
        },
      },
    ],
  },
}
