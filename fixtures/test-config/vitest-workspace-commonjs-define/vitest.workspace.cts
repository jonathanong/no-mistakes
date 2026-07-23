// CommonJS direct require is the sole default-binding namespace form accepted.
const vitest = require('vitest/config')

module.exports = vitest.defineWorkspace([{
  test: {
    name: 'commonjs-define-project',
    setupFiles: './workspace-setup.ts',
  },
}])
