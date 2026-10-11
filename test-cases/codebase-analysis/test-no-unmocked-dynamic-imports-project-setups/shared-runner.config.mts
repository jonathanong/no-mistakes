export default {
  test: {
    projects: [{ test: {
      name: 'unit',
      include: ['web/covered.test.mts'],
      setupFiles: ['./test-helpers/web.setup.mts'],
    } }],
  },
  projects: [{ name: 'e2e', testMatch: ['web/covered.test.mts'] }],
};
