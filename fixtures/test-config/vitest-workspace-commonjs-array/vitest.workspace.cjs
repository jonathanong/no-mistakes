const lookalike = [{ test: { name: 'ignored-lookalike' } }]

module.exports = [{ test: { name: 'ignored-earlier-exact' } }]

module.exports
module['exports'] = lookalike
exports.default = lookalike
other.exports = lookalike

module.exports = [{
  test: {
    name: 'commonjs-array-project',
    setupFiles: './workspace-setup.ts',
  },
}]

module.exports ||= lookalike
