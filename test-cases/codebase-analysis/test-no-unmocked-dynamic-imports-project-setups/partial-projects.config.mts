export default {
  test: { projects: [
    { test: { name: 'web', include: ['web/covered.test.mts'] } },
    { test: { name: 'node', include: ['web/covered.test.mts'] } },
  ] },
};
