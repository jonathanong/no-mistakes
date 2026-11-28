const { group: { expect: also }, test: check } = require("vitest");
const { group: { test: inner } } = require("vitest");
const { test, ...rest } = require("vitest");
check.sequential("outer key", () => {});
inner.sequential("inner key", () => {});
rest.sequential("rest key", () => {});
void also;
