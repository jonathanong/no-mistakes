const skipped = [{ test: { name: 'skipped-project' } }]

module.exports = [{ test: { name: 'commonjs-filter-project' } }]
// These expression/assignment forms deliberately follow the valid export and
// must not replace the last exact `module.exports =` workspace declaration.
skipped
module.exports += skipped
skipped = skipped
exports.default = skipped
