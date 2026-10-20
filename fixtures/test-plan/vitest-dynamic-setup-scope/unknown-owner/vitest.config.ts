const setupFiles = process.env.VITEST_SETUP

export default {
  test: {
    include: ['**/*.test.ts'],
    setupFiles,
  },
}
