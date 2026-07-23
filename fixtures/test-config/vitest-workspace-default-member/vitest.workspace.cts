// A `.default` member is not the direct CJS namespace binding.
const vitest = require('vitest/config').default

module.exports = vitest.defineWorkspace([{
  test: { name: 'unsupported-commonjs-default-member' },
}])
