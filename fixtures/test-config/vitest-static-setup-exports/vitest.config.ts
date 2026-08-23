import { cycle, files, named, spread } from './helpers'

export default {
  test: {
    setupFiles: files,
    projects: [
      {
        test: {
          name: 'spread-exported',
          setupFiles: spread,
        },
      },
      {
        test: {
          name: 'cycle-exported',
          setupFiles: cycle,
        },
      },
      {
        test: {
          name: 'named-exported',
          setupFiles: named,
        },
      },
    ],
  },
}
