module.exports = {
  projects: [{
    test: {
      name: 'cjs-named-object',
      root: './cjs-named-object-owner',
      include: ['**/*.test.ts'],
      setupFiles: './setup/named-object.ts',
      globalSetup: [],
    },
  }],
}
