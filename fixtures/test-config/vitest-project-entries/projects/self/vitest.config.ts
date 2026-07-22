export default {
  test: {
    name: 'self',
    // The same standalone config must not be parsed recursively.
    projects: ['./vitest.config.ts'],
  },
}
