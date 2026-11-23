const { expect: check } = require("@jest/globals");
const jestModule = require("@jest/globals");
const compare = jestModule.expect;

check(Date.now() - start).toBeLessThan(200);
compare(performance.now() - start).toBeGreaterThan(100);
