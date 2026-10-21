// Dynamic on purpose: the analyzer cannot resolve this statically.
const setupFiles = process.env.VITEST_SETUP

export default {
  test: {
    include: ['**/*.test.ts'],
    setupFiles,
  },
}
