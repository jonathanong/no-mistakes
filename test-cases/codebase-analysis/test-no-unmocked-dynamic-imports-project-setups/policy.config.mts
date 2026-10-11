export default {
  test: { projects: [{ test: {
    name: 'unit',
    include: ['superseded/*.test.mts'],
    setupFiles: ['./test-helpers/web.setup.mts'],
  } }] },
};
