// Root setup mocks apply only when this project inherits them.
export default { test: { setupFiles: ['./test-helpers/web.setup.mts'], projects: [
  { test: { name: 'web', include: ['web/covered.test.mts'] } },
] } };
