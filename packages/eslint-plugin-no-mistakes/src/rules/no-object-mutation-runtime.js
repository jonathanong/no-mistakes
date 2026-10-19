"use strict";

// Late-bound visitors. Helpers call these after scanMutations assigns them,
// which keeps the walker split across modules without a require cycle.
const api = {};

module.exports = api;
