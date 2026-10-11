// The top-level return is invalid in a TS module, but earlier helper facts survive.
export const helper = require('./mock.cjs');
return helper;
