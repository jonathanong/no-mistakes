exports.projects = [{ test: { name: 'cjs-named-stale' } }]
module.exports = {
  projects: [{
    test: {
      name: 'cjs-named-replacement',
      root: './cjs-named-replacement-owner',
      include: ['**/*.test.ts'],
      setupFiles: './setup/named-replacement.ts',
      globalSetup: [],
    },
  }],
}
exports.projects = [{ test: { name: 'cjs-named-detached' } }]
