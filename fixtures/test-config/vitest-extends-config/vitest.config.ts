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
        extends: './vite.config.js',
        root: '.',
        test: {
          name: 'cleared-extends',
          include: ['cleared-extends/**/*.test.ts'],
          setupFiles: [],
          globalSetup: [],
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
      {
        extends: './vite.scope.config.js',
        test: {
          name: 'scope-inherited',
          exclude: ['local-ignore/**'],
        },
      },
      {
        extends: './configs/shared/vite.cross.config.js',
        test: {
          name: 'cross-inherited',
        },
      },
      {
        extends: './configs/shared/vite.cross.config.js',
        test: {
          name: 'cross-local',
          root: './local-root',
          include: ['local/**/*.test.ts'],
          exclude: ['local-ignore/**'],
        },
      },
      {
        extends: './vite.merged.config.js',
        test: {
          name: 'merged-extends',
        },
      },
      {
        extends: './vite.merged.config.js',
        test: {
          name: 'cleared-merged-extends',
          include: ['cleared-merged-extends/**/*.test.ts'],
          setupFiles: [],
          globalSetup: [],
        },
      },
      {
        extends: './vite.merged-dynamic.config.js',
        test: {
          name: 'merged-dynamic',
          root: './merged-dynamic-root',
          include: ['**/*.test.ts'],
        },
      },
    ],
  },
}
