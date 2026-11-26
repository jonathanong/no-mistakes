var api = require("vitest");
var api = api;
const { test: fromApi } = api;
fromApi.sequential("redeclared", () => {});
