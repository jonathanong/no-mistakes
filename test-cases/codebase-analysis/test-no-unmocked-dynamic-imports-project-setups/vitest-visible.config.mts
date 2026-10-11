export default {
  test: {
    include: ['web/covered.test.mts'],
    setupFiles: ['./ignored/visible-on-disk.setup.mts'],
  },
};
