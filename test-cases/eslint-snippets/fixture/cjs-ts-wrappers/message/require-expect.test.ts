const check = require("vitest").expect;
const err = new Error("boom");
check(err.message).toBe("x");
