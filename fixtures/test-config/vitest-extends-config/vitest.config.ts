export default {
  test: {
    projects: [
      {
        extends: './vite.config.js',
        test: {
          name: 'extended',
          include: ['extended/**/*.test.ts'],
          setupFiles: './local-setup.ts',
        },
      },
      {
        extends: './cycle-base.js',
        test: {
          name: 'cycle',
          include: ['cycle/**/*.test.ts'],
        },
      },
      {
        extends: './missing-vite.config.js',
        test: {
          name: 'unresolved',
          include: ['unresolved/**/*.test.ts'],
        },
      },
      {
        extends: './vite-factory.js',
        test: {
          name: 'unsupported',
          include: ['unsupported/**/*.test.ts'],
        },
      },
    ],
  },
}
