const vt = require("vitest");
const ns = vt;
const { test: check } = ns;
check.sequential("name", () => {});
ns.test.sequential("member", () => {});
ns.sequential("namespace", () => {});
