import { standaloneImportedSetups } from './config/standalone-imported-setup-values'

export default {
  test: {
    name: 'standalone-imported-setup',
    root: './standalone-imported-setup-owner',
    include: ['**/*.test.ts'],
    setupFiles: standaloneImportedSetups,
    globalSetup: [],
  },
}
