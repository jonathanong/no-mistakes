import { setupFiles } from './setup'

export default {
  test: {
    include: ['tests/**/*.test.ts'],
    setupFiles,
  },
}
