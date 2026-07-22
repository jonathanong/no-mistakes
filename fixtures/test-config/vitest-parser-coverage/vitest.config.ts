import { firstDynamic, secondDynamic } from './dynamic'
import { directoryDynamic } from './config'

const spreadSetups = ['./setup.ts']

function namedDynamicSetup() {
  return firstDynamic || secondDynamic
}

export default {
  test: {
    projects: [
      {
        test: {
          name: 'spread-setup',
          include: ['spread/**/*.test.ts'],
          setupFiles: [, ...spreadSetups],
        },
      },
      {
        test: {
          name: 'named-dynamic',
          include: ['named/**/*.test.ts'],
          setupFiles: namedDynamicSetup,
        },
      },
      {
        test: {
          name: 'directory-dynamic',
          include: ['directory/**/*.test.ts'],
          setupFiles: directoryDynamic,
        },
      },
      './no-default.ts',
      './cycle.ts',
      './missing-project.ts',
      // The visible-path test can make this directory a candidate, exercising
      // non-readable project-config and imported-helper fallbacks.
      './config',
    ],
  },
}
