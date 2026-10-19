const setupFiles = process.env.OWNER_SETUP

export default {
  test: {
    projects: [
      {
        test: {
          name: 'owner',
          root: './owner',
          include: ['**/*.test.ts'],
          setupFiles,
        },
      },
      {
        test: {
          name: 'sibling',
          include: ['sibling/**/*.test.ts'],
        },
      },
    ],
  },
}
