const vt = require("vitest");
const key = "test";
vt[key].test.sequential("unknown member", () => {});
vt["test"].sequential("static key", () => {});
const named = require("vitest")["test"];
named.sequential("string alias", () => {});
