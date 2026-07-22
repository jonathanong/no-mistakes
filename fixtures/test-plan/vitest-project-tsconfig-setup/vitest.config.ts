export default {
  test: {
    projects: [
      {
        test: {
          name: 'unit',
          root: './packages/unit',
          include: ['tests/**/*.test.ts'],
          setupFiles: '@setup/aliased',
        },
      },
    ],
  },
}
