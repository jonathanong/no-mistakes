// The final exact assignment wins; compound assignments are not declarations.
module.exports = ['./setup/commonjs-misleading-default.ts']
module.exports = ['./setup/commonjs-default.ts', `./setup/commonjs-default-template.ts`]
module.exports += './setup/commonjs-compound-default.ts'
exports.namedSetups = ['./setup/commonjs-misleading-named.ts']
exports.namedSetups = ['./setup/commonjs-named.ts', `./setup/commonjs-named-template.ts`]
exports.namedSetups ||= ['./setup/commonjs-compound-named.ts']
// `module.exports.member` is the named-export equivalent of `exports.member`.
module.exports.moduleNamedSetups = ['./setup/commonjs-module-named.ts']
