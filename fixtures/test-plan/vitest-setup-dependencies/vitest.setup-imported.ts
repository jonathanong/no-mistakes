export default {
  test: {
    name: 'imported',
    root: './imported',
    include: ['**/*.test.ts'],
    setupFiles: ['./setup/imported.cts'],
    globalSetup: './setup/imported-global.cjs',
  },
}
