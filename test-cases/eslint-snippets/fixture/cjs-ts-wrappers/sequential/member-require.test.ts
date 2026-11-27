const run = require("vitest").test;
run.sequential("member", () => {});
let { test: kept } = require("vitest");
kept.sequential("stable let", () => {});
var { it: also } = require("vitest");
also.sequential("stable var", () => {});
