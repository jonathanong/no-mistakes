module.exports = [{
  test: {
    name: 'cjs-direct-element',
    root: './cjs-direct-element-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/direct-element.ts',
    globalSetup: [],
  },
}]
