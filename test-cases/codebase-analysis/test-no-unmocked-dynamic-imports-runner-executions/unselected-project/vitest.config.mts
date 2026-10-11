export default { test: { projects: [{ test: {
  name: 'other',
  include: ['tests/*.test.mts'],
  setupFiles: ['./other.setup.mts'],
} }] } };
