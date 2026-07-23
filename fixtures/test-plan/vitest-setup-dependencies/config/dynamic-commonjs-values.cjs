// Dynamic setup fallback retains these literal CommonJS loader dependencies.
exports.commonjsDynamicSetup = () => process.env.VITEST_SETUP
require.resolve('./dynamic-resolved-loader')
// A nonliteral loader remains unsupported and must not become a trigger.
require.resolve(process.env.DYNAMIC_RUNTIME_LOADER)
