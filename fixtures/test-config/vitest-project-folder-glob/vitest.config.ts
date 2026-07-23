export default {
  test: {
    // Configless folder projects must retain their own default include.
    include: ['root/**/*.spec.ts'],
    projects: ['packages/*'],
  },
}
