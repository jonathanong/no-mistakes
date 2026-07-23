let uninitialized
const { setupFiles } = require('./setup')
const setupSource = './setup'
const dynamicSetup = require(setupSource)

module.exports = {
  test: {
    name: 'commonjs-negative-bindings',
  },
}
