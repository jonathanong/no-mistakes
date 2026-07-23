const extendsTrue = { extends: true }
const extendsFalse = { extends: false }
import { duplicateRootGlobal, duplicateRootSetup } from './setup-values'

export default {
  test: {
    setupFiles: './root-setup.ts',
    globalSetup: './root-global.ts',
    projects: [
      { test: { name: 'default', include: ['default/**/*.test.ts'] } },
      { extends: false, test: { name: 'false', include: ['false/**/*.test.ts'] } },
      { extends: 'not-boolean', test: { name: 'nonboolean', include: ['nonboolean/**/*.test.ts'] } },
      { extends: true, test: { name: 'true', include: ['true/**/*.test.ts'] } },
      {
        extends: true,
        test: {
          name: 'cleared',
          include: ['cleared/**/*.test.ts'],
          // An empty setup array intentionally clears the inherited root setup.
          setupFiles: [],
          globalSetup: [],
        },
      },
      {
        extends: true,
        test: {
          name: 'merged-setups',
          include: ['merged/**/*.test.ts'],
          setupFiles: [duplicateRootSetup, './dir/../root-setup.ts', './project-setup.ts'],
          globalSetup: [duplicateRootGlobal, './root-global.ts', './project-global.ts'],
        },
      },
      {
        ...extendsTrue,
        ...extendsFalse,
        test: { name: 'spread-false-last', include: ['spread-false-last/**/*.test.ts'] },
      },
      {
        ...extendsFalse,
        ...extendsTrue,
        test: { name: 'spread-true-last', include: ['spread-true-last/**/*.test.ts'] },
      },
      './standalone.config.ts',
    ],
  },
}
