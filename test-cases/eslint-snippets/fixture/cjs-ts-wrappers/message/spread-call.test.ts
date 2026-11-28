const check = require("vitest").expect;
const err = new Error("boom");
check.call(...[null, 42], err.message).toBe(42);
