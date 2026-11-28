var vt = require("vitest");
vt.test.sequential("stable var", () => {});
let named = require("vitest");
named.test.sequential("stable let", () => {});
let rebound = require("vitest");
rebound = null;
rebound.test.sequential("rebound", () => {});
