let vt = require("vitest");
vt = replacement;
const ns = vt;
ns.test.sequential("alias", () => {});
const { test: suite, expect: check } = vt;
suite.sequential("destructured", () => {});
check.sequential("expect", () => {});
