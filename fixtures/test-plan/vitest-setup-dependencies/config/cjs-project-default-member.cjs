module.exports.default = [{
  test: {
    name: 'cjs-default-member',
    root: './cjs-default-member-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/default-member.ts',
    globalSetup: [],
  },
}]
