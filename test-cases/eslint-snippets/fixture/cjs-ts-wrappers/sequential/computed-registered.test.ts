let { test: run } = require("vitest");
const key = "test";
run[key].sequential("computed member", () => {});
