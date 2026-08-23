module.exports = {
  replacedObjectSetups: '../shared-setup/replaced-object-old.ts',
}
// A replacement removes the earlier named value instead of merging with it.
module.exports = {
  aliasBarrierSetups: '../shared-setup/alias-barrier-retained.ts',
  moduleOverrideSetups: '../shared-setup/module-override-original.ts',
  // A statically unrelated spread must not hide named values.
  ...{ ignoredSetups: './ignored-commonjs-object-setup.ts' },
}
// `exports` still references the pre-replacement object and cannot override it.
exports.replacedObjectSetups = '../shared-setup/replaced-object-detached.ts'
exports.aliasBarrierSetups = '../shared-setup/alias-barrier-detached.ts'
module.exports.moduleOverrideSetups = '../shared-setup/module-override.ts'
