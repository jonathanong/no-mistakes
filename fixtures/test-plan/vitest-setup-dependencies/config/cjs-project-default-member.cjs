module.exports.default = [{
  test: {
    name: 'cjs-module-default-member',
    root: './cjs-default-member-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/default-member.ts',
    globalSetup: [],
  },
}]

exports.default = [{
  test: {
    name: 'cjs-exports-default-member',
    root: './cjs-exports-default-member-owner',
    include: ['**/*.test.ts'],
    setupFiles: './setup/exports-default-member.ts',
    globalSetup: [],
  },
}]
