export default { test: {
  name: 'custom',
  include: ['custom/*.case.mts'],
  exclude: ['custom/excluded.case.mts'],
  setupFiles: ['./setup.mts'],
} };
