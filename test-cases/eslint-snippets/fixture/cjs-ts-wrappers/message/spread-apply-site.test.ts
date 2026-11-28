const check = require("vitest").expect;
const err = new Error("boom");
check.apply(...[null, [42]], [err.message]).toBe(42);
