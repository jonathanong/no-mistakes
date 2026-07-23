// Deletion removes ordinary runtime edges, so every literal require, import,
// and require.resolve loader remains a conservative setup trigger.
require('./required-helper')
import('./dynamic-helper')
require.resolve('./resolved-loader')
// Nonliteral resolution remains dynamic and must not invent a deleted trigger.
require.resolve(process.env.RUNTIME_LOADER)
