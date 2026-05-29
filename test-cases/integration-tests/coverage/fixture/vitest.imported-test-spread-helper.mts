export default {
  include: ['imported-test-spread/**/*.test.ts'],
  projects: [
    {
      test: {
        name: 'imported-test-spread',
      },
    },
  ],
}

export const namedImportedTestOptions = {
  projects: [
    {
      test: {
        name: 'named-imported-test-spread',
        include: ['named-imported-test-spread/**/*.test.ts'],
      },
    },
  ],
}
