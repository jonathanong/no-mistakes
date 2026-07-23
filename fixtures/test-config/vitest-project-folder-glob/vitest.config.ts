export default {
  test: {
    // Configless folder projects must retain their own default include.
    include: ['root/**/*.spec.ts'],
    // Intentional overlap verifies explicit configs are deduplicated from glob matches.
    projects: ['./packages/direct/vitest.config.ts', 'packages/*'],
  },
}
