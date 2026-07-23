module.exports.projects = [{
  test: {
    name: 'cjs-named-member',
    root: './cjs-named-member-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/named-member.ts',
    globalSetup: [],
  },
}]
