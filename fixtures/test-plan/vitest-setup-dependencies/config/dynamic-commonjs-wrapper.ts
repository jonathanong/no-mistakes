// This imported helper intentionally reaches its dynamic value through CJS.
const { commonjsDynamicSetup } = require('./dynamic-commonjs-values.cjs')

export const importedCommonjsDynamicSetup = commonjsDynamicSetup
