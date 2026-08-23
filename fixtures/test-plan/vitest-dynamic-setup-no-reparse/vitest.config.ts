// Dynamic expressions retain conservative setup facts and must not reparse.
const setupFiles = process.env.VITEST_SETUP

export default {
  test: {
    include: ['tests/**/*.test.ts'],
    setupFiles,
  },
}
