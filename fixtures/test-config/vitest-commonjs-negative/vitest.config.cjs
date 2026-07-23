let uninitialized
const { setupFiles } = require('./setup')
// Computed and nested bindings must remain outside the static CJS subset.
const { ['computed']: computedSetup, nested: { setupFiles: nestedSetup } } = require('./setup')
const setupSource = './setup'
const dynamicSetup = require(setupSource)

module.exports = {
  test: {
    name: 'commonjs-negative-bindings',
    // These incomplete destructuring forms must stay dynamic setup values.
    setupFiles: [computedSetup, nestedSetup],
  },
}
