module.exports = [{
  test: {
    name: 'cjs-direct-spread',
    root: './cjs-direct-spread-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/direct-spread.ts',
    globalSetup: [],
  },
}]
