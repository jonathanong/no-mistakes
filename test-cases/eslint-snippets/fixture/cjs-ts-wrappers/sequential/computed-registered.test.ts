let { test: run } = require("vitest");
const key = "test";
run[key].sequential("computed member", () => {});

const { test: check } = require("vitest");
const member = "sequential";
check[member].sequential("const computed member", () => {});
