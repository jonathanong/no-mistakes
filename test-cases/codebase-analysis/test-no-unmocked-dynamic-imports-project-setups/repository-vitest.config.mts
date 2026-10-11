export default {
  test: { projects: [{ test: {
    name: 'unit',
    include: ['web/covered.test.mts'],
    setupFiles: ['./test-helpers/web.setup.mts'],
  } }] },
};
